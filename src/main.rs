mod shaders;
mod vec3i;
mod utils;
mod world_generation;

use bevy::{prelude::*, utils::HashMap};
use vec3i::*;
use utils::*;
use world_generation::*;

const LEVEL_MIN: f32 = -300.0;
const LEVEL_MAX: f32 = 300.0;
const CAMERA_OFFSET: Vec3 = Vec3::new(-2.0, 0.0, 5.0);

#[derive(Component)]
struct Movement{
    speed: f32,
}

impl Movement {
    fn new(speed: f32) -> Self { Self { speed } }
}

#[derive(Component)]
struct Player;

#[derive(Clone)]
pub enum RootResource {
    Sap,
    Bark,
    Wood,
}

#[derive(Component)]
pub struct Root {
    id: i64,
    resource: RootResource,
    mineable: i32,
}

#[derive(Resource, Default)]
pub struct BlockMap {
    entities: HashMap<Vec3i, Entity>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<shaders::CustomMaterial>>,
    mut blockmap: ResMut<BlockMap>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0) + CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Name::new("MainCamera"),
    ));

    let cube_material = custom_materials.add(shaders::CustomMaterial { time: 0.0, bending: 0.1, cam_position: Vec3::new(-2.0, 2.5, 5.0), color: Vec3::new(1.0, 0.0, 0.0) } );
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    commands.spawn((
        MaterialMeshBundle  {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.0, 1.0, 1.0)),
            ..default()
        },
        Movement::new(1.0),
        Player,
        Name::new("Cube"),
    ));

    let sap = custom_materials.add(Color::GREEN.into());
    let bark = custom_materials.add(Color::CRIMSON.into());
    let wood = custom_materials.add(Color::BEIGE.into());

    for i in 0..1000 {
        let root_resource = random_resource(&mut rng);

        let cloned_material = match root_resource {
            RootResource::Sap => sap.clone(),
            RootResource::Bark => bark.clone(),
            RootResource::Wood => wood.clone(),
        };
        let location = random_location(&mut rng, LEVEL_MIN as i64, LEVEL_MAX as i64);

        let mut gen = RootGenerator { i, cube_mesh: &cube_mesh, cloned_material: &cloned_material, root_resource: &root_resource, rng: &mut rng, blockmap: &mut blockmap };
        gen.spawn_root(&location, &mut commands);
        gen.root_around(&location, 0.3, 0.05, &mut commands);
    }
}

fn camera_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut center = Vec3::default();
    let mut count: i32 = 0;
    for tranform in player_query.iter()  {
        center += tranform.translation;
        count += 1;
    }
    center = Vec3::new(center.x / count as f32, 0.0, center.z / count as f32);

    for mut transform in query.iter_mut() {
        center.y = transform.translation.y;
        transform.translation = center;
        if keyboard_input.pressed(KeyCode::Z) {
            transform.translation.y -= 5.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::X) {
            transform.translation.y += 5.0 * time.delta_seconds();
        }
    }
}

fn movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Movement), With<Player>>,
) {
    for (mut transform, movement) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.z -= movement.speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.z += movement.speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= movement.speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += movement.speed * time.delta_seconds();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<shaders::CustomMaterial>::default())
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .insert_resource(ClearColor(Color::GRAY))
        .insert_resource(BlockMap::default())
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(camera_system)
        .run();
}
