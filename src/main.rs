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
struct Direction(Vec3);

#[derive(Component, Default)]
struct Player{
    sap: i32,
    bark: i32,
    wood: i32,
}

#[derive(Component)]
struct CustomDamping(f32);

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

struct DamageEvent {
    target_entity: Entity,
    attacker: Entity,
    amount: i32,
}

#[derive(Resource, Default)]
pub struct BlockMap {
    entities: HashMap<Vec3i, Entity>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
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

    let ground_tex = asset_server.load("ground.png");
    let sap_tex = asset_server.load("sap.png");

    let player_material = custom_materials.add(shaders::CustomMaterial {
        time: 0.0,
        bending: 0.1,
        cam_position: Vec3::new(-2.0, 2.5, 5.0),
        color: Vec3::new(1.0, 1.0, 1.0),
        texture: asset_server.load("up.png"),
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
            mesh: plane_mesh.clone(),
            material: player_material,
            transform: Transform::from_translation(Vec3::new(1.0, 15.0, 1.0)).with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5 * PI, PI, 0.0)).with_scale(Vec3::new(1.0, 1.0, 1.5)),
            ..default()
        },
        Movement::new(30.0),
        Player::default(),
        Name::new("Cube"),
        RigidBody::Dynamic,
        Collider::ball(0.5),
        ExternalImpulse::default(),
        Velocity::default(),
        CustomDamping(0.01), // Smaller value = stronger effect
        LockedAxes::ROTATION_LOCKED,
        Direction::default(),
    ));

    let material_map: &HashMap<RootResource, Handle<CustomMaterial>> = &[
        (RootResource::Sap, custom_materials.add(CustomMaterial::new(Color::rgb(0.7, 0.7, 5.0), &sap_tex))),
        (RootResource::Bark, custom_materials.add(CustomMaterial::new(Color::CRIMSON, &ground_tex))),
        (RootResource::Wood, custom_materials.add(CustomMaterial::new(Color::BEIGE, &ground_tex))),
    ].into_iter().collect();

    let ground_material = &custom_materials.add(CustomMaterial::new(Color::DARK_GRAY, &ground_tex));

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

    // Make random bushes
    let bush_material = custom_materials.add(shaders::CustomMaterial {
        time: 0.0,
        bending: 0.1,
        cam_position: Vec3::new(-2.0, 2.5, 5.0),
        color: Vec3::new(1.0, 1.0, 1.0),
        texture: asset_server.load("bush.png"),
    });
    for _ in 0..150 {
        let location = random_location(gen.rng, LEVEL_MIN as i64, LEVEL_MAX as i64);
        commands.spawn((
            MaterialMeshBundle  {
                mesh: plane_mesh.clone(),
                material: bush_material.clone(),
                transform: Transform::from_translation(location.into()).with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5 * PI, PI, 0.0)).with_scale(Vec3::new(1.0, 1.0, 1.1)),
                ..default()
            },
            Name::new("Bush"),
        ));
    }
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
    mut query: Query<(&mut ExternalImpulse, &mut Direction, &Movement), With<Player>>,
) {
    for (mut external, mut dir, movement) in query.iter_mut() {
        // Sum vectors from directions that are pressed
        let movement_dir = KEYS
            .iter()
            .filter_map(|(key, v)| keyboard_input.pressed(*key).then_some(v))
            .sum::<Vec3>();

        if movement_dir != Vec3::ZERO {
            dir.0 = movement_dir;
        }

        external.impulse = movement_dir * movement.speed * time.delta_seconds();
    }

    // TODO: add jumping for player
}

fn custom_damping_system(
    mut query: Query<(&mut Velocity, &ExternalImpulse, &CustomDamping)>,
    time: Res<Time>,
) {
    for (mut vel, external, damping) in query.iter_mut() {
        if external.impulse == Vec3::ZERO {
            vel.linvel.x *= f32::powf(damping.0, time.delta_seconds());
            vel.linvel.z *= f32::powf(damping.0, time.delta_seconds());
        }
    }
}

fn damage_system(
    mut damage_events: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
    root_query: Query<(&Root, &BlockPosition)>,
    mut player_query: Query<&mut Player>,
    mut blockmap: ResMut<BlockMap>,
    mut commands: Commands,
) {
    for ev in damage_events.iter() {
        if let Ok(mut health) = query.get_mut(ev.target_entity) {
            health.health -= ev.amount;
            if health.health <= 0 {
                commands.entity(ev.target_entity).despawn();

                if let Ok((root, block_pos)) = root_query.get(ev.target_entity) {
                    blockmap.entities.remove_entry(&block_pos.0);

                    if let Ok(mut player) = player_query.get_mut(ev.attacker) {
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

fn collision_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(first, second, _) = collision_event {
            damage_events.send(DamageEvent { target_entity: *second, attacker: *first, amount: 1 });
        }
    }
}

fn collapse_trunks_system (
    mut query: Query<(&mut BlockPosition, &mut Transform)>,
    mut blockmap: ResMut<BlockMap>,
) {
    for (mut blocks, mut transform) in query.iter_mut() {
        let old = blocks.0;
        let new = blocks.0 + (0, -1, 0).into();
        if old.y() > 0 && !blockmap.entities.contains_key(&new) {
            blocks.0 = new;
            transform.translation = new.into();

            let entity = blockmap.entities.get(&old).copied();
            match entity {
                Some(entity) => {
                    blockmap.entities.remove_entry(&old);
                    blockmap.entities.insert(new, entity);
                }
                None => {
                    panic!("Wheres the freaking entity?!?");
                },
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

fn player_attack_system(
    query: Query<(Entity, &Transform, &Direction), With<Player>>,
    mut enemy_query: Query<Entity, (With<Health>, Without<Player>)>,
    mut damage_events: EventWriter<DamageEvent>,
    rapier_context: Res<RapierContext>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        for (player_entity, player_transform, dir) in query.iter() {
            let ray_pos = player_transform.translation;
            let ray_dir = dir.0;
            rapier_context.intersections_with_ray(ray_pos, ray_dir, 1.0, false, QueryFilter::new(),
                |entity, intersection| {
                    // Callback called on each collider hit by the ray.
                    let hit_point = intersection.point;
                    let hit_normal = intersection.normal;
                    println!("Entity {:?} hit at point {} with normal {}", entity, hit_point, hit_normal);

                    // Check if player hit anything that has Health
                    if let Ok(enemy_entity) = enemy_query.get_mut(entity) {
                        damage_events.send(DamageEvent { target_entity: enemy_entity, attacker: player_entity, amount: 1 });
                        return false;
                    }

                    true // true = continue searching
                });
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
        .add_event::<DamageEvent>()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(BlockMap::default())
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(collapse_trunks_system)
        .add_system(collision_system)
        .add_system(camera_system)
        .add_system(ui_count_system)
        .add_system(custom_damping_system)
        .add_system(player_attack_system)
        .add_system(damage_system)
        .register_type::<MainCamera>() // Only needed for in-game inspector
        .register_type::<BlockPosition>()
        .run();
}
