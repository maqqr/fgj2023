use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::MainCamera;

pub const DEFAULT_BENDING: f32 = 0.01;

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    pub time: f32,
    #[uniform(1)]
    pub bending: f32,
    #[uniform(2)]
    pub cam_position: Vec3,
    #[uniform(3)]
    pub color: Vec3,
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }
}

impl From<Color> for CustomMaterial {
    fn from(value: Color) -> Self {
        Self { time: 0.0, bending: 0.1, cam_position: Vec3::ZERO, color: Vec3::new(value.r(), value.g(), value.b()) }
    }
}

fn update_shaders(
    query: Query<(&Transform, &MainCamera)>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>,
) {
    if let Ok((camera_transform, camera)) = query.get_single() {
        for mat in custom_materials.iter_mut() {
            mat.1.time = time.elapsed_seconds();
            mat.1.cam_position = camera_transform.translation;
            mat.1.bending = if camera.bend_world { camera.bending } else { 0.0 }
        }
    }
}

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_shaders);
    }
}