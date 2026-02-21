mod vertex;
mod app;
mod vulkan;
mod swapchain;
mod vulkan_context;
mod camera;
mod buffer;
mod texture;
mod mesh;
mod scene;
mod registry;
mod bitmap;
mod gpu_mesh;
mod pipeline;
mod command;
mod depth;
mod frame_data;

use std::time::{Instant};

use app::App;

use anyhow::{Result};
use vulkanalia::vk::DeviceV1_0;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};


struct WindowHandler {
    window: Option<Window>,
    app: Option<App>,

    egui_ctx: Option<egui::Context>,
    egui_state: Option<egui_winit::State>,

    minimized: bool,
    last_frame_time: Instant,
    last_fps_time: Instant,
    frames: u32,
    camera_mode: bool,

    my_string: String,
}

impl WindowHandler {
    fn handle_event(&mut self, event_loop: &ActiveEventLoop, event: Event<()>) {
        let (window, app) = match (&self.window, &mut self.app) {
            (Some(w), Some(a)) => (w, a),
            _ => return,
        };

        match event {
            Event::AboutToWait => {
                window.request_redraw();
            },
            Event::WindowEvent { event: window_event, .. } => {
                let egui_state = self.egui_state.as_mut().unwrap();

                if egui_state.on_window_event(window, &window_event).consumed {
                    return;
                }

                match window_event {
                    WindowEvent::RedrawRequested if !event_loop.exiting() && !self.minimized => {
                        // Update camera
                        let delta_time = self.last_frame_time.elapsed();
                        self.last_frame_time = Instant::now();
                        app.camera.update(&app.camera_movement, delta_time);

                        // Test UI
                        let egui_ctx = self.egui_ctx.as_ref().unwrap();
                        let egui_state = self.egui_state.as_mut().unwrap();

                        let raw_input = egui_state.take_egui_input(window);

                        
                        let full_output = egui_ctx.run(raw_input, |ctx| {
                            // ctx.set_visuals_of(egui::Theme::Dark, egui::Visuals { window_fill: egui::Color32::RED, ..Default::default() });
                            ctx.fonts_mut(|fonts| {
                                let mut s = String::new();
                                for c in 32u8..127u8 {
                                    s.push(c as char);
                                }

                                // Force layout (this triggers rasterization)
                                fonts.layout_no_wrap(
                                    s,
                                    egui::FontId::proportional(14.0),
                                    egui::Color32::WHITE,
                                );
                            });

                            egui::Window::new("Debug").show(ctx, |ui| {
                                ui.label("Hello nigga dick");
                                ui.label(egui::RichText::new("large liutauras cock").font(egui::FontId::proportional(40.0)));
                                ui.label(egui::RichText::new("large dalius cock").font(egui::FontId::proportional(100.0)));
                                let _ = ui.button("Click");
                                let _ = ui.button("Click");
                                let _ = ui.button("Click");
                                let _ = ui.button("Click");
                                let _ = ui.button("Click");
                                let _ = ui.button("Click niggasjkdhjsakdhsakjdhaskjdhsakjdhjsad");
                                let response = ui.add(egui::TextEdit::singleline(&mut self.my_string).font(egui::FontId::proportional(14.0)));
                            });
                        });

                        egui_state.handle_platform_output(
                            window,
                            full_output.platform_output,
                        );

                        // let clipped_primitives = egui_ctx.tessellate(full_output.shapes, self.window.as_ref().unwrap().scale_factor() as f32);
                        let clipped_primitives = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        app.clipped_primitives = clipped_primitives;    
                        let textures_delta = full_output.textures_delta;
                        app.textures_delta = textures_delta;
                        // println!("suck a nigga dick, {} {}", app.textures_delta.set.len(), app.textures_delta.free.len());
                        // for (id, delta) in &full_output.textures_delta.set {
                        //     self.egui_renderer.upload_texture(*id, delta);
                        // }

                        // for x in clipped_primitives {
                        //     println!("{}", x.clip_rect);
                        // }
                        // app.set_egui_data(clipped_primitives, full_output.textures_delta);

                        // Render app
                        unsafe { app.render(window) }.expect("Rendering failed");
    
                        // Reset mouse movement to not continuously move every frame
                        app.camera_movement.rotate_horizontal = 0.0;
                        app.camera_movement.rotate_vertical = 0.0;
    
                        // Update FPS counter
                        let fps_time = self.last_fps_time.elapsed();
                        self.frames += 1;
    
                        if fps_time.as_secs_f32() > 0.25 {
                            let fps = self.frames as f64 / fps_time.as_secs_f64();
                            window.set_title(&format!("Kubilas - {:.1} FPS", fps));
                            self.last_fps_time = Instant::now();
                            self.frames = 0;
                        }
                    },
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                        unsafe {
                            if let Some(app) = self.app.take() {
                                app.vulkan_context.device.device_wait_idle().unwrap();
                                app.destroy();
                            }
                        }
                    },
                    WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state, .. }, .. } => match physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            event_loop.exit();
                            unsafe {
                                if let Some(app) = self.app.take() {
                                    app.vulkan_context.device.device_wait_idle().unwrap();
                                    app.destroy();
                                }
                            }
                        },
                        PhysicalKey::Code(KeyCode::Enter) => {
                            if state.is_pressed() {
                                self.camera_mode = !self.camera_mode;
                                
                                if self.camera_mode {
                                    window.set_cursor_visible(false);
                                    window
                                        .set_cursor_grab(CursorGrabMode::Locked)
                                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                                        .unwrap();
                                } else {
                                    window.set_cursor_visible(true);
                                    window.set_cursor_grab(CursorGrabMode::None).unwrap();
                                }
                            }
                        },
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_forward = amount;
                            }
                        },
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_left = amount;
                            }
                        },
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_backward = amount;
                            }
                        },
                        PhysicalKey::Code(KeyCode::KeyD) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_right = amount;
                            }
                        },
                        PhysicalKey::Code(KeyCode::Space) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_up = amount;
                            }
                        },
                        PhysicalKey::Code(KeyCode::ShiftLeft) => {
                            if self.camera_mode {
                                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                                app.camera_movement.amount_down = amount;
                            }
                        },
                        _ => {},
                    },
                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            self.minimized = true;
                        } else {
                            self.minimized = false;
                            app.resized = true;
                            unsafe { app.render(window) }.unwrap();
                        }
                    },
                    WindowEvent::Moved(_) if !event_loop.exiting() && !self.minimized => {
                        unsafe { app.render(window) }.unwrap();
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta: (dx, dy) }, .. } => {
                if self.camera_mode {
                    app.camera_movement.rotate_horizontal = dx as f32;
                    app.camera_movement.rotate_vertical = dy as f32;
                }
            },
            _ => {}
        }
    }
}

impl ApplicationHandler for WindowHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Kubilas")
                    .with_inner_size(LogicalSize::new(1600, 900))
            )
            .expect("Failed to create window");

        let app = unsafe { App::create(&window) }
            .expect("Failed to create app");

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        self.egui_ctx = Some(egui_ctx);
        self.egui_state = Some(egui_state);

        self.window = Some(window);
        self.app = Some(app);
        self.minimized = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        self.handle_event(event_loop, Event::WindowEvent { window_id, event });
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: winit::event::DeviceId, event: DeviceEvent) {
        self.handle_event(event_loop, Event::DeviceEvent { device_id, event });
    }
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_event(event_loop, Event::AboutToWait);
    }
}

fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new()?;
    let mut window_handler = WindowHandler { window: None, app: None, egui_ctx: None, egui_state: None, minimized: false, last_frame_time: Instant::now(), frames: 0, camera_mode: false, last_fps_time: Instant::now(), my_string: String::new() };
    event_loop.run_app(&mut window_handler)?;

    Ok(())
}
