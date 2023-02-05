use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{MainCamera, Player};

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
    #[texture(4)]
    #[sampler(5)]
    pub texture: Handle<Image>,
    #[uniform(6)]
    pub player_position: Vec3,
    #[uniform(7)]
    pub viewport_size: Vec2,
}

impl CustomMaterial {
    pub fn new(value: Color, tex: &Handle<Image>) -> Self {
        Self {
            time: 0.0,
            bending: 0.1,
            cam_position: Vec3::ZERO,
            color: Vec3::new(value.r(), value.g(), value.b()),
            texture: tex.clone(),
            player_position: Vec3::ZERO,
            viewport_size: Vec2::ZERO,
        }
    }
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }
}

fn update_shaders(
    query: Query<(&Transform, &MainCamera)>,
    player_query: Query<&Transform, With<Player>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let win_size = Vec2::new(window.width() * window.scale_factor() as f32, window.height() * window.scale_factor() as f32);

    let mut player_pos =
        if let Ok(player_t) = player_query.get_single() {
            player_t.translation
        }
        else {
            Vec3::ZERO
        };

    if let Ok((camera_transform, camera)) = query.get_single() {
        for mat in custom_materials.iter_mut() {
            mat.1.time = time.elapsed_seconds();
            mat.1.cam_position = camera_transform.translation;
            mat.1.bending = if camera.bend_world { camera.bending } else { 0.0 };
            mat.1.player_position = player_pos;
            mat.1.viewport_size = win_size;
        }
    }

    if let Ok(player_t) = player_query.get_single() {
    }
}

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_shaders);
    }
}