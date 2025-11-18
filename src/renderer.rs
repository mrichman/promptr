use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;
use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextRenderer, TextArea, Buffer, Metrics, Family, Attrs,
    Color, Shaping,
};

use crate::teleprompter::FontFamily as TeleprompterFontFamily;

use crate::teleprompter::TeleprompterState;
use crate::ui::UiState;

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    
    egui_renderer: egui_wgpu::Renderer,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        
        // Try to use Mailbox for smoother scrolling, fallback to Fifo
        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };
        
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&device, &queue, surface_format);
        let text_renderer = TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            font_system,
            swash_cache,
            atlas,
            text_renderer,
            egui_renderer,
        }
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Clamp to wgpu's maximum texture size (8192x8192)
        const MAX_TEXTURE_SIZE: u32 = 8192;
        
        let width = new_size.width.min(MAX_TEXTURE_SIZE).max(1);
        let height = new_size.height.min(MAX_TEXTURE_SIZE).max(1);
        
        if width > 0 && height > 0 {
            self.size.width = width;
            self.size.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn render(&mut self, state: &TeleprompterState, ui_state: &mut UiState, window: &Window) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => return,
        };
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0 - state.transparency as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
        
        // Prepare text buffer with centered text
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(48.0, 60.0));
        
        // Use 60% of window width for better centering
        let text_width = (self.size.width as f32 * 0.6).max(400.0);
        let left_margin = (self.size.width as f32 - text_width) / 2.0;
        
        buffer.set_size(&mut self.font_system, text_width, self.size.height as f32);
        
        let font_family = match state.font_family {
            TeleprompterFontFamily::SansSerif => Family::SansSerif,
            TeleprompterFontFamily::Serif => Family::Serif,
            TeleprompterFontFamily::Monospace => Family::Monospace,
        };
        
        buffer.set_text(&mut self.font_system, &state.text, Attrs::new().family(font_family), Shaping::Advanced);
        
        let text_area = TextArea {
            buffer: &buffer,
            left: left_margin,
            top: (self.size.height as f32 / 2.0) - state.scroll_position,
            scale: 1.0,
            bounds: glyphon::TextBounds {
                left: 0,
                top: 0,
                right: self.size.width as i32,
                bottom: self.size.height as i32,
            },
            default_color: Color::rgb(255, 255, 255),
        };
        
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                glyphon::Resolution {
                    width: self.size.width,
                    height: self.size.height,
                },
                [text_area],
                &mut self.swash_cache,
            )
            .unwrap();
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            self.text_renderer.render(&self.atlas, &mut render_pass).unwrap();
        }
        
        // Render egui UI
        let primitives = ui_state.render(window);
        
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: primitives.pixels_per_point,
        };
        
        for (id, image_delta) in &primitives.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }
        
        let paint_jobs = ui_state.egui_ctx.tessellate(primitives.shapes, primitives.pixels_per_point);
        
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        
        for id in &primitives.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        self.atlas.trim();
    }
}
