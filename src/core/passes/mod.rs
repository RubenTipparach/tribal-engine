/// Render pass plugins
///
/// Each file in this module is a self-contained rendering system

pub mod skybox;
pub mod nebula;
pub mod mesh;
pub mod star;
pub mod hologram;
pub mod outline;

pub use skybox::SkyboxPass;
pub use nebula::NebulaPass;
pub use mesh::MeshPass;
pub use star::StarPass;
pub use hologram::HolographicPass;
pub use outline::OutlinePass;
