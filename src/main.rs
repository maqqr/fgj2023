use bevy::prelude::*;
use math::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement_system)
        .run();
}

enum Movement {}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());

    let mut transform = Transform::default();
    transform.translation = Vec3::new(100.0, 100.0, 0.0);

    commands.spawn(SpriteComponents {
        material: Material::default().with_albedo(Color::rgb(1.0, 0.0, 0.0)),
        sprite: Sprite {
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..Default::default()
        },
        transform,
        movement: Movement
        ..Default::default()
    });
}

fn movement_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&Transform, &mut Translation)>) {
    for (_, mut translation) in &mut query.iter() {
        if keyboard_input.pressed(KeyCode::Up) {
            translation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            translation.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            translation.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            translation.x += 1.0;
        }

        // wrap player around screen
        if translation.x < 0.0 {
            translation.x = 199.0;
        }
        if translation.x > 200.0 {
            translation.x = 0.0;
        }
        if translation.y < 0.0 {
            translation.y = 199.0;
        }
        if translation.y > 200.0 {
            translation.y = 0.0;
        }
    }
}