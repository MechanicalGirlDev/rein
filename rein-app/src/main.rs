use std::sync::Arc;

use glam::Vec3;
use rein_engine::ecs::components::rendering::{
    CameraComponent, FrustumCullable, LightComponent, MaterialHandle, MeshHandle, MeshRenderer,
    Visible,
};
use rein_engine::ecs::components::transform::{GlobalTransform, Transform};
use rein_engine::engine::{App, GameLoopConfig, SystemContext, run_app};
use rein_engine::renderer::light::LightType;
use rein_engine::{Camera, ColorMaterial, Mesh, WgpuContext, WindowSettings};

struct EditorApp {
    mesh_spawned: bool,
}

impl App for EditorApp {
    fn init(&mut self, _ctx: &WgpuContext, world: &mut hecs::World) {
        // Camera
        let camera = Camera::new_perspective(
            Vec3::new(3.0, 3.0, 3.0),
            Vec3::ZERO,
            Vec3::Y,
            45.0,
            1.0,
            0.1,
            100.0,
        );
        world.spawn((
            Transform::identity(),
            GlobalTransform::default(),
            CameraComponent {
                camera,
                active: true,
            },
        ));

        // Light
        world.spawn((
            Transform::from_position(Vec3::new(5.0, 5.0, 5.0)),
            GlobalTransform::default(),
            LightComponent {
                light_type: LightType::Directional,
                color: Vec3::ONE,
                intensity: 1.0,
            },
        ));
    }

    fn update(&mut self, world: &mut hecs::World, ctx: &SystemContext) {
        // Spawn mesh on first update (surface_format is available here)
        if !self.mesh_spawned {
            let material = ColorMaterial::new(ctx.ctx, ctx.surface_format)
                .expect("Failed to create material");
            let mesh = Mesh::cube(ctx.ctx, 1.0, [0.8, 0.3, 0.2]);

            world.spawn((
                Transform::identity(),
                GlobalTransform::default(),
                MeshRenderer {
                    mesh: MeshHandle(Arc::new(mesh)),
                    material: MaterialHandle(Arc::new(material)),
                    visible: true,
                    cast_shadow: true,
                    receive_shadow: true,
                },
                FrustumCullable,
                Visible,
            ));
            self.mesh_spawned = true;
        }

        // Update camera viewport
        for (_, (cam,)) in world.query_mut::<(&mut CameraComponent,)>() {
            if cam.active {
                cam.camera.set_viewport(ctx.viewport);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let settings = WindowSettings::default().title("Rein Editor");
    let config = GameLoopConfig::default();
    let app = EditorApp {
        mesh_spawned: false,
    };
    run_app(settings, config, app)
}
