use egui_winit::egui;
use winit::window::Window;

use crate::teleprompter::{TeleprompterState, FontFamily};

pub struct UiState {
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub show_controls: bool,
}

impl UiState {
    pub fn new(window: &Window) -> Self {
        let egui_ctx = egui::Context::default();
        
        // Set larger font sizes and spacing
        let mut style = (*egui_ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(28.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(22.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
        ].into();
        
        // Increase spacing and sizes
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 12.0);
        style.spacing.slider_width = 300.0;
        style.spacing.interact_size = egui::vec2(60.0, 40.0);
        
        egui_ctx.set_style(style);
        
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
        );
        
        Self {
            egui_ctx,
            egui_state,
            show_controls: true,
        }
    }
    
    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        self.egui_state.on_window_event(window, event)
    }
    
    pub fn update(&mut self, window: &Window, teleprompter: &mut TeleprompterState) {
        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_ctx.begin_frame(raw_input);
        
        if self.show_controls {
            egui::Window::new("Controls")
                .default_pos([20.0, 20.0])
                .default_width(400.0)
                .resizable(true)
                .show(&self.egui_ctx, |ui| {
                    ui.heading("Teleprompter Controls");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(12.0);
                    
                    ui.label("Scroll Speed:");
                    ui.add_space(4.0);
                    ui.add(egui::Slider::new(&mut teleprompter.speed, 10.0..=200.0)
                        .suffix(" px/s")
                        .text_color(egui::Color32::WHITE));
                    
                    ui.add_space(16.0);
                    
                    ui.label("Background Opacity:");
                    ui.add_space(4.0);
                    ui.add(egui::Slider::new(&mut teleprompter.transparency, 0.0..=1.0)
                        .show_value(true)
                        .text_color(egui::Color32::WHITE));
                    
                    ui.add_space(16.0);
                    
                    ui.label("Font Family:");
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_source("font_family")
                        .selected_text(teleprompter.font_family.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut teleprompter.font_family, FontFamily::SansSerif, "Sans Serif");
                            ui.selectable_value(&mut teleprompter.font_family, FontFamily::Serif, "Serif");
                            ui.selectable_value(&mut teleprompter.font_family, FontFamily::Monospace, "Monospace");
                        });
                    
                    ui.add_space(20.0);
                    
                    // Larger buttons
                    let button_size = egui::vec2(180.0, 50.0);
                    
                    if ui.add_sized(button_size, egui::Button::new(
                        if teleprompter.paused { "Resume" } else { "Pause" }
                    )).clicked() {
                        teleprompter.toggle_pause();
                    }
                    
                    ui.add_space(8.0);
                    
                    if ui.add_sized(button_size, egui::Button::new("Reset")).clicked() {
                        teleprompter.reset();
                    }
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    ui.label("Keyboard Shortcuts:");
                    ui.add_space(4.0);
                    ui.label("SPACE - Pause/Resume");
                    ui.label("UP/DOWN - Speed");
                    ui.label("F - Fullscreen");
                    ui.label("T - Toggle Transparency");
                    ui.label("H - Hide/Show Controls");
                    ui.label("ESC - Exit");
                });
        }
    }
    
    pub fn render(&mut self, window: &Window) -> egui::FullOutput {
        let full_output = self.egui_ctx.end_frame();
        self.egui_state.handle_platform_output(
            window,
            full_output.platform_output.clone(),
        );
        
        full_output
    }
}
