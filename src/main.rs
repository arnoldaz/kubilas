#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
)]

mod vertex;
mod app;
mod vulkan;
mod swapchain;
mod image;
mod cpu_render_object;
mod gpu_render_object;
mod validations;
mod camera;

use std::time::{Instant};

use app::App;

use anyhow::{Result};
use vulkanalia::vk::DeviceV1_0;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

// NEXT: https://kylemayes.github.io/vulkanalia/vertex/staging_buffer.html

// #[derive(Debug)]
struct WindowHandler {
    window: Option<Window>,
    app: Option<App>,
    minimized: bool,
    last_frame_time: Instant,
    frames: u32,
    canmera_mode: bool,
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
            }

            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::RedrawRequested if !event_loop.exiting() && !self.minimized => {
                    unsafe { app.render(window) }.expect("Rendering failed");

                    let now = Instant::now();
                    let elapsed = now.duration_since(self.last_frame_time);
                    self.frames += 1;

                    if elapsed.as_secs_f32() > 0.25 {
                        let fps = self.frames as f64 / elapsed.as_secs_f64();
                        window.set_title(&format!("Kubilas - {:.1} FPS", fps));
                        self.last_frame_time = now;
                        self.frames = 0;
                    }
                }

                WindowEvent::CloseRequested => {
                    event_loop.exit();
                    unsafe {
                        app.device.device_wait_idle().unwrap();
                        app.destroy();
                    }
                }

                WindowEvent::KeyboardInput {
                    event: KeyEvent { physical_key, state: ElementState::Pressed, .. },
                    ..
                } => match physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                        unsafe {
                            app.device.device_wait_idle().unwrap();
                            app.destroy();
                        }
                    },
                    PhysicalKey::Code(KeyCode::Enter) => {
                        self.canmera_mode = !self.canmera_mode;

                        if self.canmera_mode {
                            window.set_cursor_visible(false);
                            window
                                .set_cursor_grab(CursorGrabMode::Locked)
                                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                                .unwrap();
                        } else {
                            window.set_cursor_visible(true);
                            window.set_cursor_grab(CursorGrabMode::None).unwrap();
                        }
                    },
                    _ => {},
                }

                WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        self.minimized = true;
                    } else {
                        self.minimized = false;
                        app.resized = true;
                        unsafe { app.render(window) }.unwrap();
                    }
                }

                WindowEvent::Moved(_) if !event_loop.exiting() && !self.minimized => {
                    unsafe { app.render(window) }.unwrap();
                }

                _ => {}
            },

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if self.canmera_mode {
                    app.update_camera(delta.0 as f32, delta.1 as f32, 1.0);
                }
                // println!("{} {} delta", delta.0, delta.1);
            }

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
                    .with_inner_size(LogicalSize::new(1024, 768))
            )
            .expect("Failed to create window");

        let app = unsafe { App::create(&window) }
            .expect("Failed to create app");

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
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    let mut window_handler = WindowHandler { window: None, app: None, minimized: false, last_frame_time: Instant::now(), frames: 0, canmera_mode: false };
    event_loop.run_app(&mut window_handler)?;

    Ok(())
}
