use core::f32;

use bevy::prelude::*;
use rand::prelude::*;

const levelMin: f32 = -1000.0;
const levelMax: f32 = 1000.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement_system)
        .run();
}

#[derive(Component)]
struct Movement;

enum RootResource {
    Sap,
    Bark,
    Wood
}

#[derive(Component)]
struct Root
{
    id: i64,
    resource: RootResource
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Name::new("MainCamera"),
    ));
    let cube_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            transform: Transform::from_translation(Vec3::new(1.0, 1.0, 1.0)),
            ..default()
        },
        Movement,
        Name::new("Cube"),
    ));

    for i in 0..100 {
        let root_resource = random_resource();

        let color = match root_resource {
            RootResource::Sap => Color::GREEN,
            RootResource::Bark => Color::CRIMSON,
            RootResource::Wood => Color::BEIGE,
            _ => Color::BLACK,
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    color,
                    ..Default::default()
                },
                transform: Transform::from_translation(random_location(levelMin, levelMax)),
                ..default()
            },
            Root { id: i, resource: root_resource},
        ));
    }
}

fn random_location(min: f32, max: f32) -> Vec3 {
    Vec3 { x: (generate_random_between(min, max)), y: 0.0, z: (generate_random_between(min, max)) }
}

fn random_resource() -> RootResource {
    let rng = generate_random_number();
    if rng > 0.8 {
        return RootResource::Sap;
    }
    if rng > 0.5 {
        return RootResource::Bark;
    }
    RootResource::Wood
}

fn generate_random_between(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let range = min..max;
    rng.gen_range(range)
}

fn generate_random_number() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>()
}

fn movement_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Movement>>) {
    for (mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 1.0;
        }
    }
}
