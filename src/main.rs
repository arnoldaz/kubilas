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
mod validations;
mod camera;
mod buffer;
mod texture;
mod mesh;
mod scene;
mod registry;
mod bitmap;
mod gpu_mesh;

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

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

// NEXT: https://kylemayes.github.io/vulkanalia/vertex/staging_buffer.html

// #[derive(Debug)]
struct WindowHandler {
    window: Option<Window>,
    app: Option<App>,
    minimized: bool,
    last_frame_time: Instant,
    last_fps_time: Instant,
    frames: u32,
    camera_mode: bool,
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
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::RedrawRequested if !event_loop.exiting() && !self.minimized => {
                    // Update camera
                    let delta_time = self.last_frame_time.elapsed();
                    self.last_frame_time = Instant::now();
                    app.camera.update(&app.camera_movement, delta_time);

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
                        app.device.device_wait_idle().unwrap();
                        app.destroy();
                    }
                },
                WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state, .. }, .. } => match physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                        unsafe {
                            app.device.device_wait_idle().unwrap();
                            app.destroy();
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
    let mut window_handler = WindowHandler { window: None, app: None, minimized: false, last_frame_time: Instant::now(), frames: 0, camera_mode: false, last_fps_time: Instant::now() };
    event_loop.run_app(&mut window_handler)?;

    Ok(())
}
