use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    dpi::LogicalSize,
    platform::scancode::PhysicalKeyExtScancode,
};

mod renderer;
mod teleprompter;
mod ui;

use renderer::Renderer;
use teleprompter::TeleprompterState;
use ui::UiState;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Teleprompter")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_transparent(true)
            .with_decorations(true)
            .build(&event_loop)
            .unwrap()
    );

    let mut renderer = pollster::block_on(Renderer::new(window.clone()));
    let mut teleprompter = TeleprompterState::new();
    let mut ui_state = UiState::new(&window);
    let mut is_fullscreen = false;

    let _ = event_loop.run(move |event, elwt| {
        // Target 60 FPS for smooth scrolling
        let target_frame_time = std::time::Duration::from_millis(16);
        elwt.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + target_frame_time
        ));

        match &event {
            Event::WindowEvent { event, .. } => {
                // Let egui handle the event first
                let response = ui_state.handle_event(&window, event);
                
                // Only process non-UI events if egui didn't consume them
                if !response.consumed {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        
                        WindowEvent::Resized(size) => {
                            renderer.resize(*size);
                        }
                        
                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state == ElementState::Pressed {
                                if let Some(scancode) = event.physical_key.to_scancode() {
                                    match scancode {
                                        49 => teleprompter.toggle_pause(), // Space
                                        126 => teleprompter.increase_speed(), // Up
                                        125 => teleprompter.decrease_speed(), // Down
                                        3 => { // F
                                            is_fullscreen = !is_fullscreen;
                                            if is_fullscreen {
                                                // Workaround for winit 0.29 macOS bug: avoid monitor APIs
                                                window.set_decorations(false);
                                                window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));
                                                // Use max texture size (8192) to avoid wgpu limits
                                                let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(8192, 8192));
                                            } else {
                                                window.set_decorations(true);
                                                let _ = window.request_inner_size(LogicalSize::new(800, 600));
                                            }
                                        }
                                        17 => teleprompter.toggle_transparency(), // T
                                        15 => teleprompter.reset(), // R
                                        4 => ui_state.show_controls = !ui_state.show_controls, // H
                                        53 => elwt.exit(), // Escape
                                        _ => {}
                                    }
                                }
                            }
                        }
                        
                        _ => {}
                    }
                } else {
                    // egui consumed the event, handle special cases
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => {
                            renderer.resize(*size);
                        }
                        _ => {}
                    }
                }
                
                // Always handle redraw
                if matches!(event, WindowEvent::RedrawRequested) {
                    // Update teleprompter state before rendering
                    teleprompter.update();
                    ui_state.update(&window, &mut teleprompter);
                    renderer.render(&teleprompter, &mut ui_state, &window);
                }
            },
            
            Event::AboutToWait => {
                // Continuously request redraws for smooth animation
                window.request_redraw();
            }
            
            _ => {}
        }
    });
}
