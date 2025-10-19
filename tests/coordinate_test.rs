/// Tests to verify screen-to-world coordinate transformations
/// This ensures raycast picking matches rendered object positions

use glam::{Mat4, Vec3, Vec4};

/// Project a 3D world point to screen coordinates using the same math as the renderer
fn world_to_screen(
    world_pos: Vec3,
    view: Mat4,
    proj: Mat4,
    viewport_width: f32,
    viewport_height: f32,
) -> (f32, f32) {
    // Transform to clip space
    let clip_pos = proj * view * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);

    // Perspective divide to NDC
    let ndc = Vec3::new(
        clip_pos.x / clip_pos.w,
        clip_pos.y / clip_pos.w,
        clip_pos.z / clip_pos.w,
    );

    // NDC to screen coordinates
    let screen_x = (ndc.x + 1.0) * 0.5 * viewport_width;
    let screen_y = (ndc.y + 1.0) * 0.5 * viewport_height;

    (screen_x, screen_y)
}

/// Unproject screen coordinates to a ray direction (current implementation)
fn screen_to_ray_current(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    view: Mat4,
    proj: Mat4,
) -> Vec3 {
    // Current implementation
    let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
    let ndc_y = (2.0 * mouse_y) / viewport_height - 1.0;

    let ray_clip = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);

    let inv_proj = proj.inverse();
    let ray_view = inv_proj * ray_clip;
    let ray_view = Vec4::new(ray_view.x, ray_view.y, -1.0, 0.0);

    let inv_view = view.inverse();
    let ray_world = inv_view * ray_view;

    Vec3::new(ray_world.x, ray_world.y, ray_world.z).normalize()
}

#[test]
fn test_coordinate_roundtrip() {
    // Set up camera and viewport
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let aspect = viewport_width / viewport_height;

    // Camera looking at origin from (0, 0, 5)
    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
    let view = Mat4::look_at_rh(
        camera_pos,
        Vec3::ZERO,
        Vec3::Y,
    );

    // Vulkan-style projection with flipped Y
    let mut proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    proj.y_axis.y *= -1.0;

    // Test point at origin (should be center of screen)
    let world_point = Vec3::ZERO;
    let (screen_x, screen_y) = world_to_screen(world_point, view, proj, viewport_width, viewport_height);

    println!("World point {:?} -> Screen ({}, {})", world_point, screen_x, screen_y);
    println!("Expected center: ({}, {})", viewport_width / 2.0, viewport_height / 2.0);

    // Test if screen center projects back to a ray pointing at origin
    let ray_dir = screen_to_ray_current(
        viewport_width / 2.0,
        viewport_height / 2.0,
        viewport_width,
        viewport_height,
        view,
        proj,
    );

    println!("Screen center -> Ray direction: {:?}", ray_dir);
    println!("Expected direction: {:?}", (world_point - camera_pos).normalize());

    // The ray should point forward (-Z direction in camera space)
    let expected_dir = (world_point - camera_pos).normalize();
    let dot = ray_dir.dot(expected_dir);
    println!("Dot product (should be close to 1.0): {}", dot);

    assert!(dot > 0.99, "Ray direction doesn't match expected direction. Dot product: {}", dot);
}

#[test]
fn test_corner_coordinates() {
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let aspect = viewport_width / viewport_height;

    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);

    let mut proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    proj.y_axis.y *= -1.0;

    // Test corners
    let corners = vec![
        ("Top-left", 0.0, 0.0),
        ("Top-right", viewport_width, 0.0),
        ("Bottom-left", 0.0, viewport_height),
        ("Bottom-right", viewport_width, viewport_height),
        ("Center", viewport_width / 2.0, viewport_height / 2.0),
    ];

    for (name, x, y) in corners {
        let ray_dir = screen_to_ray_current(x, y, viewport_width, viewport_height, view, proj);
        println!("{} ({}, {}) -> Ray: {:?}", name, x, y, ray_dir);
    }
}

#[test]
fn test_y_axis_arrow_projection() {
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let aspect = viewport_width / viewport_height;

    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);

    let mut proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    proj.y_axis.y *= -1.0;

    // Y-axis arrow tip (pointing up)
    let y_arrow_tip = Vec3::new(0.0, 1.0, 0.0);
    let (screen_x, screen_y) = world_to_screen(y_arrow_tip, view, proj, viewport_width, viewport_height);

    println!("Y-axis arrow tip (0, 1, 0) -> Screen ({}, {})", screen_x, screen_y);

    // Should be above center (smaller Y in screen space means higher up)
    let center_y = viewport_height / 2.0;
    println!("Center Y: {}, Arrow Y: {}", center_y, screen_y);

    // In screen space, Y=0 is top, Y=height is bottom
    // So arrow pointing up should have smaller screen_y than center
    if screen_y < center_y {
        println!("✓ Arrow is above center (correct)");
    } else {
        println!("✗ Arrow is below center (incorrect)");
    }

    // NOW TEST THE REVERSE: Does a ray from the arrow's screen position point at the arrow?
    let ray_dir = screen_to_ray_current(screen_x, screen_y, viewport_width, viewport_height, view, proj);
    println!("\nReverse test:");
    println!("Casting ray from screen ({}, {})...", screen_x, screen_y);
    println!("Ray direction: {:?}", ray_dir);

    // The ray should point in the direction from camera to arrow
    let expected_dir = (y_arrow_tip - camera_pos).normalize();
    println!("Expected direction (camera->arrow): {:?}", expected_dir);

    let dot = ray_dir.dot(expected_dir);
    println!("Dot product: {} (should be close to 1.0)", dot);

    assert!(dot > 0.95, "Reverse projection failed! Screen ({}, {}) -> Ray doesn't point at arrow. Dot: {}",
            screen_x, screen_y, dot);
}
