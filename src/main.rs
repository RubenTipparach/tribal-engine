mod engine;
mod renderer;
mod mesh;
mod lighting;
mod game;
mod imgui_renderer;
mod background;
mod ui;
mod nebula;

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
