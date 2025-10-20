mod core;
mod engine;
mod mesh;
mod material;
mod material_library;
mod game;
mod game_manager;
mod imgui_renderer;
mod background;
mod ui;
mod nebula;
mod config;
mod scene;
mod gizmo;
mod ecs;      // ECS system with 64-bit coordinates
mod movement; // Turn-based movement system

use engine::Engine;

fn main() -> anyhow::Result<()> {
    println!("=== Tribal Engine Starting ===");
    println!("Initializing Vulkan renderer...");
    let engine = Engine::new()?;
    println!("Engine initialized successfully!");
    println!("Starting render loop...");
    engine.run()?;
    println!("Engine shutdown complete.");
    Ok(())
}
