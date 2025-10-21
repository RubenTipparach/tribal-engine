/// Render pass plugins
///
/// Each file in this module is a self-contained rendering system

pub mod skybox;
pub mod nebula;
pub mod mesh;
pub mod star;
pub mod outline;
pub mod line;
pub mod unlit;

pub use skybox::SkyboxPass;
pub use nebula::NebulaPass;
pub use mesh::MeshPass;
pub use star::StarPass;
pub use outline::OutlinePass;
pub use line::LinePass;
pub use unlit::UnlitPass;
