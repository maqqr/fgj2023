#[macro_use]
extern crate lazy_static;

mod shaders;
mod vec3i;
mod utils;
mod world_generation;
mod constants;

use std::f32::consts::PI;

use bevy::{prelude::*, utils::HashMap, core_pipeline::bloom::BloomSettings};
use bevy_rapier3d::prelude::*;
use shaders::CustomMaterial;
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

#[derive(Component, Default)]
struct Player{
    sap: i32,
    bark: i32,
    wood: i32,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCamera {
    bend_world: bool,
    bending: f32,
    offset: Vec3,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

#[derive(Component)]
pub struct Health {
    health: i32
}

#[derive(Resource, Default)]
pub struct BlockMap {
    entities: HashMap<Vec3i, Entity>,
}

#[derive(Component)]
pub struct BlockPosition(Vec3i);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<shaders::CustomMaterial>>,
    mut blockmap: ResMut<BlockMap>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_translation(INITIAL_CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 500.0,
                fov: FIELD_OF_VIEW,
                ..default()
            }),
            ..default()
        },
        Name::new("MainCamera"),
        MainCamera { bend_world: true, bending: DEFAULT_BENDING, offset: INITIAL_CAMERA_OFFSET },
        BloomSettings { threshold: 0.6, knee: 0.2, intensity: 0.15, ..default() },
        UiCameraConfig { show_ui: true },
    ));

    let test_tex = asset_server.load("test.png");

    let cube_material = custom_materials.add(shaders::CustomMaterial {
        time: 0.0,
        bending: 0.1,
        cam_position: Vec3::new(-2.0, 2.5, 5.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        texture: test_tex.clone(),
    });
    commands.spawn(
        TextBundle::from_section(
            format_ui_text(0,0,0),
            TextStyle {
                font: asset_server.load("monogram.ttf"),
                font_size: 30.0,
                color: Color::WHITE,

            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(5.0)),
            position: UiRect {
                left: Val::Px(210.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
    );

    let cube_mesh = &meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let plane_mesh = &meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    // Create player entity
    commands.spawn((
        MaterialMeshBundle  {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.0, 15.0, 1.0)),
            ..default()
        },
        Movement::new(10.0),
        Player::default(),
        Name::new("Cube"),
        RigidBody::Dynamic,
        Collider::ball(0.5),
        ExternalImpulse::default(),
    ));

    let material_map: &HashMap<RootResource, Handle<CustomMaterial>> = &[
        (RootResource::Sap, custom_materials.add(CustomMaterial::new(Color::GREEN, &test_tex))),
        (RootResource::Bark, custom_materials.add(CustomMaterial::new(Color::CRIMSON, &test_tex))),
        (RootResource::Wood, custom_materials.add(CustomMaterial::new(Color::BEIGE, &test_tex))),
    ].into_iter().collect();

    let ground_material = &custom_materials.add(CustomMaterial::new(Color::DARK_GRAY, &test_tex));

    let height_chances = [0.05, 0.1, 0.2, 0.3, 0.2, 0.05, 0.03, 0.03, 0.02, 0.02];

    let mut gen = WorldGenerator { cube_mesh, plane_mesh, material_map, ground_material, rng: &mut rng, blockmap: &mut blockmap, height_chances: &height_chances };
    for i in 0..500 {
        let location = random_location(gen.rng, LEVEL_MIN as i64, LEVEL_MAX as i64);
        if gen.blockmap.entities.contains_key(&location) {
            continue;
        }
        let root_resource = random_resource(gen.rng);

        gen.spawn_root_block(i, &location, root_resource, &mut commands);
        gen.root_around(i, &location, root_resource, 0.3, 0.05, &mut commands);

        gen.make_trunk(i, &location, root_resource, 12,&mut commands);
    }
    gen.make_ground_plane(&mut commands);
}

fn format_ui_text(sap: i32, bark: i32, wood: i32) -> String {
    format!("Sap: {sap}\nBark: {bark}\nWood:{wood}")
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
    center = Vec3::new(center.x / count as f32, center.y / count as f32, center.z / count as f32);

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

    // TODO: add jumping for player
}

fn collision_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(&mut Health, &Root, &BlockPosition)>,
    mut player_query: Query<&mut Player>,
    mut blockmap: ResMut<BlockMap>,
    mut commands: Commands,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(first, second, _) = collision_event {
            if let Ok((mut health, root, block_position)) = query.get_mut(*second) {
                health.health -= 1;
                if health.health <= 0 {
                    blockmap.entities.remove_entry(&block_position.0);
                    commands.entity(*second).despawn();
                    
                    if let Ok(mut player) = player_query.get_mut(*first) {
                        match root.resource {
                            RootResource::Sap => player.sap += root.mineable,
                            RootResource::Bark => player.bark += root.mineable,
                            RootResource::Wood => player.wood += root.mineable,
                        }
                    }
                }
            }
        }   
    }
}

fn collapse_trunks_system (
    mut query: Query<(&mut BlockPosition, &mut Transform)>,
    mut blockmap: ResMut<BlockMap>,
) {
    for (mut blocks, mut transform) in query.iter_mut() {
        let old = blocks.0;
        let new = Vec3i::new(blocks.0.x(), blocks.0.y() - 1, blocks.0.z());
        if !blockmap.entities.contains_key(&new) {
            let y = blocks.0.y();
            blocks.0.set_y(y - 1);
            transform.translation = blocks.0.into();

            let entity = blockmap.entities.get(&old);
            match entity {
                Some(entity) => {
                    blockmap.entities.remove_entry(&old);
                    blockmap.entities.insert(new, *entity);
                }
                None => panic!("Wheres the freaking entity?!?"),
            }

        }
    }
}

fn ui_count_system (
    mut query: Query<&mut Text>, 
    player_query: Query<&Player, Changed<Player>>,
) {
    for player in &player_query {
        for mut text in query.iter_mut() {
            text.sections.first_mut().unwrap().value = format_ui_text(player.sap, player.bark, player.wood)
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<shaders::CustomMaterial>::default())
        // .add_plugin(bevy_editor_pls::EditorPlugin)
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(shaders::ShaderPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(BlockMap::default())
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(collapse_trunks_system)
        .add_system(collision_system)
        .add_system(camera_system)
        .add_system(ui_count_system)
        .register_type::<MainCamera>() // Only needed for in-game inspector
        .run();
}
