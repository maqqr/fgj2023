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

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,) {
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
}

fn movement_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Movement>>) {
    for (mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 0.01;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= 0.01;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 0.01;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 0.01;
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
