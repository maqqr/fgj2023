#[macro_use]
extern crate lazy_static;

mod constants;
mod shaders;
mod utils;
mod vec3i;
mod world_generation;

use std::{f32::consts::PI};

use bevy::{
    audio::*,
    core_pipeline::bloom::BloomSettings,
    prelude::*,
    utils::HashMap,
};
use bevy_rapier3d::prelude::*;
use constants::*;
use rand::rngs::ThreadRng;
use shaders::CustomMaterial;
use utils::*;
use vec3i::*;
use world_generation::*;

#[derive(Component)]
struct Movement {
    speed: f32,
}

impl Movement {
    fn new(speed: f32) -> Self {
        Self { speed }
    }
}

#[derive(Component, Default)]
struct Direction(Vec3);

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default)]
enum CardinalDirection {
    #[default] Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Default)]
struct Player {
    sap: i32,
    bark: i32,
    wood: i32,
    last_direction: CardinalDirection,
    images: HashMap<CardinalDirection, Handle<Image>>,
    strike_images: HashMap<CardinalDirection, Handle<Image>>,
    strike_anim_timer: f32,
}

#[derive(Component)]
struct CustomDamping(f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCamera {
    bend_world: bool,
    bending: f32,
    offset: Vec3,
    shake_intensity: f32,
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
    health: i32,
}

struct DamageEvent {
    target_entity: Entity,
    attacker: Entity,
    amount: i32,
}

struct AnimEvent {
    direction: CardinalDirection,
    is_strike: bool,
}

#[derive(Resource, Default)]
pub struct BlockMap {
    entities: HashMap<Vec3i, Entity>,
}

#[derive(Resource, Default)]
pub struct AudioHandles {
    sap: Handle<AudioSource>,
    bark: Handle<AudioSource>,
    wood: Handle<AudioSource>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BlockPosition(Vec3i);

#[derive(Component)]
struct Particle {
    lifetime_left: f32,
    velocity: Vec3,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<shaders::CustomMaterial>>,
    mut blockmap: ResMut<BlockMap>,
    asset_server: Res<AssetServer>,
    mut audioHandles: ResMut<AudioHandles>,
    audio: Res<Audio>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_translation(INITIAL_CAMERA_OFFSET)
                .looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                far: 500.0,
                fov: FIELD_OF_VIEW,
                ..default()
            }),
            ..default()
        },
        Name::new("MainCamera"),
        MainCamera {
            bend_world: true,
            bending: DEFAULT_BENDING,
            offset: INITIAL_CAMERA_OFFSET,
            shake_intensity: 10.0,
        },
        BloomSettings {
            threshold: 0.6,
            knee: 0.2,
            intensity: 0.3,
            ..default()
        },
        UiCameraConfig { show_ui: true },
    ));

    let ground_tex = asset_server.load("ground.png");
    let sap_tex = asset_server.load("sap.png");
    let bark_tex = asset_server.load("bark.png");
    let wood_tex = asset_server.load("wood.png");

    let sap_sound: Handle<AudioSource> = asset_server.load("SapFast.ogg");
    let wood_sound: Handle<AudioSource> = asset_server.load("Wood.ogg");
    let bark_sound: Handle<AudioSource> = asset_server.load("Bark.ogg");
    let music: Handle<AudioSource> = asset_server.load("DheJamas.ogg");
    audioHandles.sap = sap_sound;
    audioHandles.wood = wood_sound;
    audioHandles.bark = bark_sound;

    let player_material = custom_materials.add(shaders::CustomMaterial {
        time: 0.0,
        bending: 0.1,
        cam_position: Vec3::new(-2.0, 2.5, 5.0),
        color: Vec3::new(1.0, 1.0, 1.0),
        texture: asset_server.load("up.png"),
        player_position: Vec3::ZERO,
        viewport_size: Vec2::ZERO,
    });
    commands.spawn(
        TextBundle::from_section(
            format_ui_text(0, 0, 0),
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
        }),
    );

    let cube_mesh = &meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let plane_mesh = &meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    let mut player_images = HashMap::new();
    player_images.insert(CardinalDirection::Left, asset_server.load("right.png"));
    player_images.insert(CardinalDirection::Right, asset_server.load("left.png"));
    player_images.insert(CardinalDirection::Up, asset_server.load("up.png"));
    player_images.insert(CardinalDirection::Down, asset_server.load("down.png"));

    let mut player_strike_images = HashMap::new();
    player_strike_images.insert(CardinalDirection::Left, asset_server.load("right_s.png"));
    player_strike_images.insert(CardinalDirection::Right, asset_server.load("left_s.png"));
    player_strike_images.insert(CardinalDirection::Up, asset_server.load("up_s.png"));
    player_strike_images.insert(CardinalDirection::Down, asset_server.load("down_s.png"));

    // Create player entity
    commands.spawn((
        MaterialMeshBundle {
            mesh: plane_mesh.clone(),
            material: player_material,
            transform: Transform::from_translation(Vec3::new(1.0, 15.0, 1.0))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5 * PI, PI, 0.0))
                .with_scale(Vec3::new(1.0, 1.0, 1.5)),
            ..default()
        },
        Movement::new(30.0),
        Player {
            images: player_images,
            strike_images: player_strike_images,
            ..default()
        },
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
        (
            RootResource::Sap,
            custom_materials.add(CustomMaterial::new(Color::rgb(1.0, 1.0, 10.0), &sap_tex)),
        ),
        (
            RootResource::Bark,
            custom_materials.add(CustomMaterial::new(Color::WHITE, &bark_tex)),
        ),
        (
            RootResource::Wood,
            custom_materials.add(CustomMaterial::new(Color::WHITE, &wood_tex)),
        ),
    ]
    .into_iter()
    .collect();

    let ground_material = &custom_materials.add(CustomMaterial::new(Color::WHITE, &ground_tex));

    let height_chances = [0.1, 0.4, 0.7, 0.85, 0.95, 0.96, 0.98, 0.99];

    let mut gen = WorldGenerator {
        cube_mesh,
        plane_mesh,
        material_map,
        ground_material,
        rng: &mut rng,
        blockmap: &mut blockmap,
        height_chances: &height_chances,
    };
    const ROOT_CHANCE: f32 = 0.3;
    const ROOT_GROWTH: f32 = 0.1;

    for i in 0..100 {
        let location = random_location(gen.rng, LEVEL_MIN as i64, LEVEL_MAX as i64);
        if gen.blockmap.entities.contains_key(&location) {
            continue;
        }
        let root_resource = RootResource::Sap;

        gen.make_trunk(i, &location, root_resource, 18, 5, ROOT_CHANCE, ROOT_GROWTH, &mut commands);
    }
    gen.make_ground_plane(&mut commands);

    // Make random bushes
    let bush_material = custom_materials.add(shaders::CustomMaterial {
        time: 0.0,
        bending: 0.1,
        cam_position: Vec3::new(-2.0, 2.5, 5.0),
        color: Vec3::new(1.0, 1.0, 1.0),
        texture: asset_server.load("bush.png"),
        player_position: Vec3::ZERO,
        viewport_size: Vec2::ZERO,
    });
    for _ in 0..350 {
        let location = random_location(gen.rng, LEVEL_MIN as i64, LEVEL_MAX as i64);
        commands.spawn((
            MaterialMeshBundle {
                mesh: plane_mesh.clone(),
                material: bush_material.clone(),
                transform: Transform::from_translation(location.into())
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5 * PI, PI, 0.0))
                    .with_scale(Vec3::new(1.0, 1.0, 1.1)),
                ..default()
            },
            Name::new("Bush"),
        ));
    }

    audio.play_with_settings(music, PlaybackSettings { repeat: true, volume: 0.5, ..default() });
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
    let mut rng = rand::thread_rng();

    let mut center = Vec3::default();
    let mut count: i32 = 0;
    for tranform in player_query.iter() {
        center += tranform.translation;
        count += 1;
    }
    center = Vec3::new(
        center.x / count as f32,
        center.y / count as f32,
        center.z / count as f32,
    );

    for (mut transform, mut camera) in query.iter_mut() {

        let shake = generate_random_unit_vec(&mut rng) * camera.shake_intensity;

        transform.translation = center + camera.offset + shake;

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
    mut query: Query<(&mut ExternalImpulse, &mut Direction, &Movement, &Transform, &mut Player)>,
    not_player_query: Query<Entity, Without<Player>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    rapier_context: Res<RapierContext>,
    mut anim_events: EventWriter<AnimEvent>,
) {
    let ray_hit = |pos, dir| {
        let mut hit = false;
        rapier_context.intersections_with_ray(
            pos,
            dir,
            1.0,
            false,
            QueryFilter::new(),
            |entity, _| {
                if not_player_query.get(entity).is_ok() {
                    hit = true;
                    return false;
                }
                true // true = continue searching
            },
        );
        hit
    };

    for (mut external, mut dir, movement, transform, mut player) in query.iter_mut() {
        // Sum vectors from directions that are pressed
        let movement_dir = KEYS
            .iter()
            .filter_map(|(key, v)| keyboard_input.pressed(*key).then_some(v))
            .sum::<Vec3>()
            .normalize_or_zero();

        if movement_dir != Vec3::ZERO {
            dir.0 = movement_dir;
        }
        external.impulse = movement_dir * movement.speed * time.delta_seconds();

        // Jumping
        if keyboard_input.just_pressed(KeyCode::Space)
            && ray_hit(transform.translation, (0.0, -1.0, 0.0).into())
        {
            external.impulse += Vec3::new(0.0, 4.0, 0.0);
        }

        let cdir =
            if movement_dir == (0.0, 0.0, -1.0).into() {
                Some(CardinalDirection::Up)
            }
            else if movement_dir == (0.0, 0.0, 1.0).into() {
                Some(CardinalDirection::Down)
            }
            else if movement_dir == (-1.0, 0.0, 0.0).into() {
                Some(CardinalDirection::Left)
            }
            else if movement_dir == (1.0, 0.0, 0.0).into() {
                Some(CardinalDirection::Right)
            }
            else {
                None
            };

        if let Some(dir) = cdir {
            player.last_direction = dir;
            anim_events.send(AnimEvent { direction: dir, is_strike: false });
        }
    }
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
    mut camera_query: Query<&mut MainCamera>,
    mut blockmap: ResMut<BlockMap>,
    audio_handles: Res<AudioHandles>,
    audio: Res<Audio>,
    mut commands: Commands,
) {
    for ev in damage_events.iter() {
        if let Ok(mut health) = query.get_mut(ev.target_entity) {
            health.health -= ev.amount;

            camera_query.single_mut().shake_intensity += 0.02;

            let root_tuple = root_query.get(ev.target_entity);

            if let Ok((root, _)) = root_tuple {
                match root.resource {
                    RootResource::Sap => audio.play(audio_handles.sap.clone()),
                    RootResource::Bark => audio.play(audio_handles.bark.clone()),
                    RootResource::Wood => audio.play(audio_handles.wood.clone()),
                };
            }

            if health.health <= 0 {
                commands.entity(ev.target_entity).despawn();
                camera_query.single_mut().shake_intensity += 0.1;

                if let Ok((root, block_pos)) = root_tuple {
                    blockmap.entities.remove_entry(&block_pos.0);

                    if let Ok(mut player) = player_query.get_mut(ev.attacker) {
                        match root.resource {
                            RootResource::Sap => {
                                player.sap += root.mineable;
                                audio.play(audio_handles.sap.clone());
                            }
                            RootResource::Bark => {
                                player.bark += root.mineable;
                                audio.play(audio_handles.bark.clone());
                            }
                            RootResource::Wood => player.wood += root.mineable,
                        }
                    }
                }
            }
        }
    }
}

fn collision_system(
    root_query: Query<(&Root)>,
    mut collision_events: EventReader<CollisionEvent>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(first, second, _) = collision_event {
            if let Ok(root) = root_query.get(*second) {
                if root.resource == RootResource::Sap {
                    damage_events.send(DamageEvent {
                        target_entity: *second,
                        attacker: *first,
                        amount: 1,
                    });
                }
            }
        }
    }
}

fn collapse_trunks_system(
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
                    print!("Wheres the freaking entity?!?");
                }
            }
        }
    }
}

fn ui_count_system(mut query: Query<&mut Text>, player_query: Query<&Player, Changed<Player>>) {
    for player in &player_query {
        for mut text in query.iter_mut() {
            text.sections.first_mut().unwrap().value =
                format_ui_text(player.sap, player.bark, player.wood)
        }
    }
}

fn player_attack_system(
    query: Query<(Entity, &Transform, &Direction, &Player)>,
    mut enemy_query: Query<Entity, (With<Health>, Without<Player>)>,
    mut damage_events: EventWriter<DamageEvent>,
    mut anim_events: EventWriter<AnimEvent>,
    rapier_context: Res<RapierContext>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        for (player_entity, player_transform, dir, player) in query.iter() {

            anim_events.send(AnimEvent { direction: player.last_direction, is_strike: true });

            let ray_pos = player_transform.translation;
            let ray_dir = dir.0;
            rapier_context.intersections_with_ray(
                ray_pos,
                ray_dir,
                1.0,
                false,
                QueryFilter::new(),
                |entity, intersection| {
                    // Callback called on each collider hit by the ray.
                    let hit_point = intersection.point;
                    let hit_normal = intersection.normal;

                    // Check if player hit anything that has Health
                    if let Ok(enemy_entity) = enemy_query.get_mut(entity) {
                        damage_events.send(DamageEvent {
                            target_entity: enemy_entity,
                            attacker: player_entity,
                            amount: 1,
                        });
                        return false;
                    }

                    true // true = continue searching
                },
            );
        }
    }
}

fn animation_system(
    mut query: Query<(&Handle<CustomMaterial>, &mut Player)>,
    mut anim_events: EventReader<AnimEvent>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>,
) {
    for ev in anim_events.iter() {
        for (mat, mut player) in query.iter_mut() {
            if let Some(mat) = custom_materials.get_mut(mat) {

                if ev.is_strike {
                    player.strike_anim_timer = 0.2;
                    mat.texture = player.strike_images.get(&ev.direction).unwrap().clone();
                }
                else {
                    mat.texture = player.images.get(&ev.direction).unwrap().clone();
                }

            }
        }
    }

    for (mat, mut player) in query.iter_mut() {
        if player.strike_anim_timer > 0.0 {
            player.strike_anim_timer -= time.delta_seconds();

            if player.strike_anim_timer <= 0.0 {
                if let Some(mat) = custom_materials.get_mut(mat) {
                    if let Some(img) = player.images.get(&player.last_direction) {
                        mat.texture = img.clone();
                    }
                }
            }
        }
    }
}

fn camera_shake_system(mut query: Query<&mut MainCamera>, time: Res<Time>) {
    for mut camera in query.iter_mut() {
        camera.shake_intensity += -camera.shake_intensity * 10.0 * time.delta_seconds();
    }
}

fn particle_system(mut query: Query<(&mut Particle, &mut Transform)>) {

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
        .add_event::<AnimEvent>()
        .insert_resource(ClearColor(Color::rgb(27.0 / 255.0, 28.0 / 255.0, 17.0 / 255.0)))
        .insert_resource(BlockMap::default())
        .insert_resource(AudioHandles::default())
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(collapse_trunks_system)
        .add_system(collision_system)
        .add_system(camera_system)
        .add_system(ui_count_system)
        .add_system(custom_damping_system)
        .add_system(player_attack_system)
        .add_system(damage_system)
        .add_system(animation_system)
        .add_system(camera_shake_system)
        .register_type::<MainCamera>() // Only needed for in-game inspector
        .register_type::<BlockPosition>()
        .run();
}
