#[macro_use]
extern crate lazy_static;

mod shaders;
mod vec3i;
mod utils;
mod world_generation;
mod constants;

use bevy::{prelude::*, utils::HashMap, core_pipeline::bloom::BloomSettings};
use bevy_rapier3d::prelude::*;
use vec3i::*;
use utils::*;
use world_generation::*;
use constants::*;

#[derive(Component)]
struct Movement{
    speed: f32,
}

impl Movement {
    fn new(speed: f32) -> Self { Self { speed } }
}

#[derive(Component)]
struct Player;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCamera {
    bend_world: bool,
    bending: f32,
    offset: Vec3,
}

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
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_translation(INITIAL_CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Name::new("MainCamera"),
        MainCamera { bend_world: true, bending: DEFAULT_BENDING, offset: INITIAL_CAMERA_OFFSET },
        BloomSettings { threshold: 0.6, knee: 0.2, intensity: 0.15, ..default() },
    ));

    let cube_material = custom_materials.add(shaders::CustomMaterial { time: 0.0, bending: 0.1, cam_position: Vec3::new(-2.0, 2.5, 5.0), color: Vec3::new(1.0, 0.0, 0.0) } );
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    // Create player entity
    commands.spawn((
        MaterialMeshBundle  {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.0, 1.0, 1.0)),
            ..default()
        },
        Movement::new(10.0),
        Player,
        Name::new("Cube"),
        RigidBody::Dynamic,
        Collider::ball(0.5),
        ExternalImpulse::default(),
    ));

    // Create invisible ground for debugging purposes
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
        Collider::cuboid(100.0, 0.1, 100.0),
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
    mut query: Query<(&mut Transform, &mut MainCamera), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut center = Vec3::default();
    let mut count: i32 = 0;
    for tranform in player_query.iter()  {
        center += tranform.translation;
        count += 1;
    }
    center = Vec3::new(center.x / count as f32, 0.0, center.z / count as f32);

    for (mut transform, mut camera) in query.iter_mut() {
        transform.translation = center + camera.offset;

        let dir = (center - transform.translation).normalize();
        if keyboard_input.pressed(KeyCode::Z) {
            camera.offset += dir * 5.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::X) {
            camera.offset -= dir * 5.0 * time.delta_seconds();
        }
    }
}

fn movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut ExternalImpulse, &Movement), With<Player>>,
) {
    for (mut external, movement) in query.iter_mut() {
        // Sum vectors from directions that are pressed
        let movement_dir = KEYS
            .iter()
            .filter_map(|(key, v)| keyboard_input.pressed(*key).then_some(v))
            .sum::<Vec3>();

        external.impulse = movement_dir * movement.speed * time.delta_seconds();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<shaders::CustomMaterial>::default())
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(shaders::ShaderPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(BlockMap::default())
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(camera_system)
        .register_type::<MainCamera>() // Only needed for in-game inspector
        .run();
}
