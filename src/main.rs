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
mod widget_gallery;

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

use crate::widget_gallery::WidgetGallery;


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

    widget_gallery: WidgetGallery,
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
                            
                            let mut open = true;
                            WidgetGallery::show(&mut self.widget_gallery, &ctx, &mut open);

                            egui::Window::new("My Window")
                                .id(egui::Id::new("my_window"))
                                .resizable(true)
                                .scroll([true, true])
                                .show(&ctx, |ui| {


                                    // ui.label("Welcome to the widget gallery!");
                                    // ui.end_row();

                                    // use egui::special_emojis::GITHUB;
                                    // ui.hyperlink_to(
                                    //     format!("{GITHUB} egui on GitHub"),
                                    //     "https://github.com/emilk/egui",
                                    // );
                                    // ui.end_row();

                                    // ui.add(egui::TextEdit::singleline(&mut self.my_string).hint_text("Write something here"));
                                    // ui.end_row();

                                    // if ui.button("Click me!").clicked() {
                                    //     self.my_bool = !self.my_bool;
                                    // }
                                    // ui.end_row();

                                    // if ui.link("Click me!").clicked() {
                                    //     self.my_bool = !self.my_bool;
                                    // }
                                    // ui.end_row();

                                    // ui.checkbox(&mut self.my_bool, "Checkbox");
                                    // ui.end_row();

                                    // ui.horizontal(|ui| {
                                    //     ui.radio_value(&mut self.my_enum, MyEnum::First, "First");
                                    //     ui.radio_value(&mut self.my_enum, MyEnum::Second, "Second");
                                    //     ui.radio_value(&mut self.my_enum, MyEnum::Third, "Third");
                                    // });
                                    // ui.end_row();

                                    // ui.horizontal(|ui| {
                                    //     ui.selectable_value(&mut self.my_enum, MyEnum::First, "First");
                                    //     ui.selectable_value(&mut self.my_enum, MyEnum::Second, "Second");
                                    //     ui.selectable_value(&mut self.my_enum, MyEnum::Third, "Third");
                                    // });
                                    // ui.end_row();


                                    // egui::ComboBox::from_label("Take your pick")
                                    //     .show_ui(ui, |ui| {
                                    //         ui.selectable_value(&mut self.my_enum, MyEnum::First, "First");
                                    //         ui.selectable_value(&mut self.my_enum, MyEnum::Second, "Second");
                                    //         ui.selectable_value(&mut self.my_enum, MyEnum::Third, "Third");
                                    //     });
                                    // ui.end_row();

                                    // ui.add(egui::Slider::new(&mut self.my_float, 0.0..=360.0).suffix("°"));
                                    // ui.end_row();

                                    // ui.add(egui::DragValue::new(&mut self.my_float).speed(1.0));
                                    // ui.end_row();

                                    // let progress = self.my_float / 360.0;
                                    // let progress_bar = egui::ProgressBar::new(progress as f32)
                                    //     .show_percentage()
                                    //     .animate(self.my_bool);
                                    // self.my_bool = ui
                                    //     .add(progress_bar)
                                    //     .on_hover_text("The progress bar can be animated!")
                                    //     .hovered();
                                    // ui.end_row();

                                    // ui.color_edit_button_srgba(&mut self.my_color);
                                    // ui.end_row();

                                    // let egui_icon = egui::include_image!("icon.png");
                                    // ui.add(egui::Image::new(egui_icon.clone()));
                                    // ui.end_row();
                                    // if ui
                                    //     .add(egui::Button::image_and_text(egui_icon, "Click me!"))
                                    //     .clicked()
                                    // {
                                    //     self.my_bool = !self.my_bool;
                                    // }
                                    // ui.end_row();

                                    // ui.separator();
                                    // ui.end_row();

                                    // ui.collapsing("Click to see what is hidden!", |ui| {
                                    //     ui.horizontal_wrapped(|ui| {
                                    //         ui.spacing_mut().item_spacing.x = 0.0;
                                    //         ui.label("It's a ");
    
                                    //         ui.add_space(4.0);
                                    //         ui.add(egui::Spinner::new());
                                    //     });
                                    // });
                                    // ui.end_row();


                                });
                        

                        
                        });

                        egui_state.handle_platform_output(
                            window,
                            full_output.platform_output,
                        );

                        let clipped_primitives = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        app.clipped_primitives = clipped_primitives;    
                        let textures_delta = full_output.textures_delta;
                        app.textures_delta = textures_delta;
                        if app.textures_delta.set.len() > 0 || app.textures_delta.free.len() > 0 {
                            println!("suck a nigga dick, {} {}", app.textures_delta.set.len(), app.textures_delta.free.len());
                        }

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

        // let mut fonts = egui::FontDefinitions::default();

        // fonts.font_data.insert(
        //     "noto".into(),
        //     std::sync::Arc::new(egui::FontData::from_static(include_bytes!("NotoSans-Regular.ttf"))),
        // );

        // fonts.families.get_mut(&egui::FontFamily::Proportional)
        //     .unwrap()
        //     .insert(0, "noto".into());

        // egui_ctx.set_fonts(fonts);

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
    let mut window_handler = WindowHandler {
        window: None,
        app: None,
        egui_ctx: None,
        egui_state: None,
        minimized: false,
        last_frame_time: Instant::now(),
        frames: 0,
        camera_mode: false,
        last_fps_time: Instant::now(),
        widget_gallery: Default::default(),
    };
    event_loop.run_app(&mut window_handler)?;

    Ok(())
}
