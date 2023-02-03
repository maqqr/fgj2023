use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement_system)
        .run();
}

#[derive(Component)]
struct Movement;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let mut transform = Transform::default();
    transform.translation = Vec3::new(100.0, 100.0, 0.0);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },
            transform,
            ..default()
        },
        Movement,
    ));
}

fn movement_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Movement>>) {
    for (mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 1.0;
        }

        // wrap player around screen
        if transform.translation.x < 0.0 {
            transform.translation.x = 199.0;
        }
        if transform.translation.x > 200.0 {
            transform.translation.x = 0.0;
        }
        if transform.translation.y < 0.0 {
            transform.translation.y = 199.0;
        }
        if transform.translation.y > 200.0 {
            transform.translation.y = 0.0;
        }
    }
}
