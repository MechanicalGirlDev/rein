//! Bridge between existing Gm<G,M> pattern and ECS.

use std::sync::Arc;

use crate::ecs::components::rendering::{
    FrustumCullable, MaterialHandle, MeshHandle, MeshRenderer, Visible,
};
use crate::ecs::components::transform::{GlobalTransform, Transform};
use crate::renderer::geometry::Geometry;
use crate::renderer::material::Material;
use crate::renderer::object::Gm;

/// Marker trait for materials that can be shared in the ECS via Arc.
///
/// Existing Material implementations using wgpu types (RenderPipeline, BindGroup, Buffer)
/// are all Send + Sync, so this blanket impl covers them automatically.
pub trait MaterialResource: Material + Send + Sync {}

impl<T: Material + Send + Sync> MaterialResource for T {}

/// Spawn a `Gm<G, M>` as an ECS entity.
///
/// Creates an entity with Transform, GlobalTransform, MeshRenderer,
/// FrustumCullable, and Visible components.
pub fn spawn_gm<G, M>(world: &mut hecs::World, gm: Gm<G, M>) -> hecs::Entity
where
    G: Geometry + Send + Sync + 'static,
    M: Material + Send + Sync + 'static,
{
    let transform = Transform::from_matrix(gm.transform);
    let global = GlobalTransform(gm.transform);
    let renderer = MeshRenderer {
        mesh: MeshHandle(Arc::new(gm.geometry)),
        material: MaterialHandle(Arc::new(gm.material)),
        visible: true,
        cast_shadow: true,
        receive_shadow: true,
    };

    world.spawn((transform, global, renderer, FrustumCullable, Visible))
}
