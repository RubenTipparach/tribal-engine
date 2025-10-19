use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState, DeviceEvent, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};
use crate::core::renderer::VulkanRenderer;
use crate::game::Game;
use crate::ui::UiManager;
use std::collections::HashSet;

pub struct Engine {
    event_loop: EventLoop<()>,
    renderer: VulkanRenderer,
}

struct GameState {
    game: Game,
    last_update_time: std::time::Instant,
    pressed_keys: HashSet<KeyCode>,
    mouse_delta: (f64, f64),
    mouse_position: (f64, f64),
    right_mouse_pressed: bool,
    left_mouse_pressed: bool,
    middle_mouse_pressed: bool,
    camera_speed: f32,
    frame_count: u32,
    fps_timer: std::time::Instant,
    current_fps: f32,
}

impl Engine {
    pub fn new() -> anyhow::Result<Self> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Tribal Engine - Vulkan SDF Renderer")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)?;

        let renderer = VulkanRenderer::new(window)?;

        Ok(Self {
            event_loop,
            renderer,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut game = Game::new();

        // Load configs and scene from files
        UiManager::load_all_configs(&mut game);
        UiManager::load_scene_on_startup(&mut game);

        let now = std::time::Instant::now();
        let mut game_state = GameState {
            game,
            last_update_time: now,
            pressed_keys: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
            right_mouse_pressed: false,
            left_mouse_pressed: false,
            middle_mouse_pressed: false,
            camera_speed: 5.0,
            frame_count: 0,
            fps_timer: now,
            current_fps: 0.0,
        };

        // Show cursor by default so user can interact with ImGui
        self.renderer.window().set_cursor_visible(true);

        self.event_loop.run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            // Pass all events to ImGui first
            let window_ptr = self.renderer.window() as *const _;
            let window = unsafe { &*window_ptr };
            self.renderer.handle_imgui_event(window, &event);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let old_mouse = game_state.mouse_position;
                    game_state.mouse_position = (position.x, position.y);

                    // Handle gizmo drag if active
                    if game_state.game.gizmo_state.using_gizmo && !self.renderer.imgui_wants_mouse() {
                        let (viewport_width, viewport_height) = self.renderer.viewport_size();
                        game_state.game.handle_mouse_drag(
                            (old_mouse.0 as f32, old_mouse.1 as f32),
                            (position.x as f32, position.y as f32),
                            viewport_width,
                            viewport_height,
                        );
                    }
                    // Update hover state if not using ImGui or gizmo
                    else if !self.renderer.imgui_wants_mouse() && !game_state.game.gizmo_state.using_gizmo {
                        let (viewport_width, viewport_height) = self.renderer.viewport_size();
                        game_state.game.handle_mouse_hover(
                            position.x as f32,
                            position.y as f32,
                            viewport_width,
                            viewport_height,
                        );
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    // Handle left mouse for object selection and gizmo interaction
                    if button == MouseButton::Left {
                        match state {
                            ElementState::Pressed => {
                                if !self.renderer.imgui_wants_mouse() {
                                    let (viewport_width, viewport_height) = self.renderer.viewport_size();
                                    game_state.game.handle_mouse_click(
                                        game_state.mouse_position.0 as f32,
                                        game_state.mouse_position.1 as f32,
                                        viewport_width,
                                        viewport_height,
                                    );
                                }
                            }
                            ElementState::Released => {
                                game_state.game.handle_mouse_release();
                            }
                        }
                    }

                    // Only handle right mouse button for camera control if ImGui isn't using the mouse
                    if button == MouseButton::Right && !self.renderer.imgui_wants_mouse() {
                        match state {
                            ElementState::Pressed => {
                                game_state.right_mouse_pressed = true;
                                self.renderer.window().set_cursor_visible(false);
                                let _ = self.renderer.window().set_cursor_grab(winit::window::CursorGrabMode::Confined);
                            }
                            ElementState::Released => {
                                game_state.right_mouse_pressed = false;
                                self.renderer.window().set_cursor_visible(true);
                                let _ = self.renderer.window().set_cursor_grab(winit::window::CursorGrabMode::None);
                                // Reset mouse delta when releasing button
                                game_state.mouse_delta = (0.0, 0.0);
                            }
                        }
                    }

                    if button == MouseButton::Middle && !self.renderer.imgui_wants_mouse() {
                        match state {
                            ElementState::Pressed => {
                                game_state.middle_mouse_pressed = true;
                                self.renderer.window().set_cursor_visible(false);
                                let _ = self.renderer.window().set_cursor_grab(winit::window::CursorGrabMode::Confined);
                            }
                            ElementState::Released => {
                                game_state.middle_mouse_pressed = false;
                                self.renderer.window().set_cursor_visible(true);
                                let _ = self.renderer.window().set_cursor_grab(winit::window::CursorGrabMode::None);
                                // Reset mouse delta when releasing button
                                game_state.mouse_delta = (0.0, 0.0);
                            }
                        }
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // Accumulate mouse delta when right or middle mouse button is pressed
                    if game_state.right_mouse_pressed || game_state.middle_mouse_pressed {
                        game_state.mouse_delta.0 += delta.0;
                        game_state.mouse_delta.1 += delta.1;
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            physical_key: PhysicalKey::Code(key_code),
                            state,
                            ..
                        },
                        ..
                    },
                    ..
                } => {
                    match state {
                        ElementState::Pressed => {
                            game_state.pressed_keys.insert(key_code);

                            // Gizmo mode hotkeys (1, 2, 3) - only if not typing in ImGui
                            if !self.renderer.imgui_wants_keyboard() {
                                match key_code {
                                    KeyCode::Digit1 => {
                                        game_state.game.gizmo_state.mode = crate::gizmo::GizmoMode::Translate;
                                    }
                                    KeyCode::Digit2 => {
                                        game_state.game.gizmo_state.mode = crate::gizmo::GizmoMode::Rotate;
                                    }
                                    KeyCode::Digit3 => {
                                        game_state.game.gizmo_state.mode = crate::gizmo::GizmoMode::Scale;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        ElementState::Released => {
                            game_state.pressed_keys.remove(&key_code);
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseWheel { delta, .. },
                    ..
                } => {
                    use winit::event::MouseScrollDelta;
                    let scroll_amount = match delta {
                        MouseScrollDelta::LineDelta(_x, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => (pos.y / 20.0) as f32,
                    };
                    game_state.camera_speed = (game_state.camera_speed + scroll_amount).max(0.1).min(50.0);
                    println!("Camera Speed: {:.1}", game_state.camera_speed);
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.renderer.handle_resize();
                }
                Event::AboutToWait => {
                    self.renderer.window().request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Update game logic
                    let now = std::time::Instant::now();
                    let delta_time = now.duration_since(game_state.last_update_time).as_secs_f32();
                    game_state.last_update_time = now;

                    // Process input
                    process_input(&mut game_state, delta_time);

                    game_state.game.update(delta_time);

                    // Update FPS counter
                    game_state.frame_count += 1;
                    let elapsed = game_state.fps_timer.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        game_state.current_fps = game_state.frame_count as f32 / elapsed;
                        game_state.frame_count = 0;
                        game_state.fps_timer = std::time::Instant::now();
                    }

                    // Update window title with FPS and dirty indicator
                    let dirty_indicator = if game_state.game.is_dirty() { " *" } else { "" };
                    let title = format!("Tribal Engine - {:.0} FPS{}", game_state.current_fps, dirty_indicator);
                    self.renderer.window().set_title(&title);

                    // Render with game state
                    if let Err(e) = self.renderer.render(&mut game_state.game) {
                        eprintln!("Render error: {}", e);
                        target.exit();
                    }
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}

fn process_input(game_state: &mut GameState, delta_time: f32) {
    // Mouse camera controls
    let mouse_sensitivity = 0.002;

    // Right mouse - free camera rotation
    if game_state.right_mouse_pressed && (game_state.mouse_delta.0 != 0.0 || game_state.mouse_delta.1 != 0.0) {
        game_state.game.rotate_camera(
            -(game_state.mouse_delta.1 as f32) * mouse_sensitivity,  // Pitch (vertical)
            -(game_state.mouse_delta.0 as f32) * mouse_sensitivity,  // Yaw (horizontal)
        );
        game_state.mouse_delta = (0.0, 0.0);
    }

    // Middle mouse - orbit around selected object
    if game_state.middle_mouse_pressed && (game_state.mouse_delta.0 != 0.0 || game_state.mouse_delta.1 != 0.0) {
        game_state.game.orbit_camera_around_selected(
            -(game_state.mouse_delta.1 as f32) * mouse_sensitivity,  // Pitch (vertical)
            -(game_state.mouse_delta.0 as f32) * mouse_sensitivity,  // Yaw (horizontal)
        );
        game_state.mouse_delta = (0.0, 0.0);
    }

    // Free camera movement controls
    let speed = game_state.camera_speed * delta_time;

    // W/S - Forward/Backward (in the direction camera is facing)
    if game_state.pressed_keys.contains(&KeyCode::KeyW) {
        game_state.game.move_camera_forward(speed);
    }
    if game_state.pressed_keys.contains(&KeyCode::KeyS) {
        game_state.game.move_camera_forward(-speed);
    }

    // A/D - Strafe left/right
    if game_state.pressed_keys.contains(&KeyCode::KeyA) {
        game_state.game.move_camera_right(-speed);
    }
    if game_state.pressed_keys.contains(&KeyCode::KeyD) {
        game_state.game.move_camera_right(speed);
    }

    // Q/E - Roll
    if game_state.pressed_keys.contains(&KeyCode::KeyQ) {
        game_state.game.roll_camera(-2.0 * delta_time);
    }
    if game_state.pressed_keys.contains(&KeyCode::KeyE) {
        game_state.game.roll_camera(2.0 * delta_time);
    }

    // Skybox tweaking controls
    let config_speed = 0.5 * delta_time;

    // Star density (1/2)
    if game_state.pressed_keys.contains(&KeyCode::Digit1) {
        game_state.game.skybox_config.star_density = (game_state.game.skybox_config.star_density - config_speed).max(0.1);
        println!("Star Density: {:.2}", game_state.game.skybox_config.star_density);
    }
    if game_state.pressed_keys.contains(&KeyCode::Digit2) {
        game_state.game.skybox_config.star_density = (game_state.game.skybox_config.star_density + config_speed).min(2.0);
        println!("Star Density: {:.2}", game_state.game.skybox_config.star_density);
    }

    // Star brightness (3/4)
    if game_state.pressed_keys.contains(&KeyCode::Digit3) {
        game_state.game.skybox_config.star_brightness = (game_state.game.skybox_config.star_brightness - config_speed).max(0.0);
        println!("Star Brightness: {:.2}", game_state.game.skybox_config.star_brightness);
    }
    if game_state.pressed_keys.contains(&KeyCode::Digit4) {
        game_state.game.skybox_config.star_brightness = (game_state.game.skybox_config.star_brightness + config_speed).min(3.0);
        println!("Star Brightness: {:.2}", game_state.game.skybox_config.star_brightness);
    }

    // Nebula intensity (5/6)
    if game_state.pressed_keys.contains(&KeyCode::Digit5) {
        game_state.game.skybox_config.nebula_intensity = (game_state.game.skybox_config.nebula_intensity - config_speed * 0.5).max(0.0);
        println!("Nebula Intensity: {:.2}", game_state.game.skybox_config.nebula_intensity);
    }
    if game_state.pressed_keys.contains(&KeyCode::Digit6) {
        game_state.game.skybox_config.nebula_intensity = (game_state.game.skybox_config.nebula_intensity + config_speed * 0.5).min(2.0);
        println!("Nebula Intensity: {:.2}", game_state.game.skybox_config.nebula_intensity);
    }

    // Background brightness (7/8)
    if game_state.pressed_keys.contains(&KeyCode::Digit7) {
        game_state.game.skybox_config.background_brightness = (game_state.game.skybox_config.background_brightness - config_speed * 0.1).max(0.0);
        println!("Background Brightness: {:.2}", game_state.game.skybox_config.background_brightness);
    }
    if game_state.pressed_keys.contains(&KeyCode::Digit8) {
        game_state.game.skybox_config.background_brightness = (game_state.game.skybox_config.background_brightness + config_speed * 0.1).min(0.5);
        println!("Background Brightness: {:.2}", game_state.game.skybox_config.background_brightness);
    }

    // Print controls help (H key)
    if game_state.pressed_keys.contains(&KeyCode::KeyH) {
        println!("\n=== SKYBOX CONTROLS ===");
        println!("1/2: Star Density (-/+)");
        println!("3/4: Star Brightness (-/+)");
        println!("5/6: Nebula Intensity (-/+)");
        println!("7/8: Background Brightness (-/+)");
        println!("H: Show this help");
        println!("=======================\n");
    }
}
