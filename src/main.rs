use core::f32;

use bevy::prelude::*;
use rand::prelude::*;

const LEVEL_MIN: f32 = -300.0;
const LEVEL_MAX: f32 = 300.0;

#[derive(Component)]
struct Movement{
    speed: f32
}

impl Movement {
    fn new(speed: f32) -> Self { Self { speed } }
}

#[derive(Component)]
struct Player;

#[derive(Clone)]
enum RootResource {
    Sap,
    Bark,
    Wood
}

impl RootResource {
    
}

#[derive(Component)]
struct Root
{
    id: i64,
    resource: RootResource
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::GREEN))
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(camera_system)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 50.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        Movement::new(1.0),
        Player,
        Name::new("Cube"),
    ));

    let sap = materials.add(Color::GREEN.into());
    let bark = materials.add(Color::CRIMSON.into());
    let wood = materials.add(Color::BEIGE.into());

    for i in 0..100 {
        let root_resource = random_resource();

        let cloned_material = match root_resource {
            RootResource::Sap => sap.clone(),
            RootResource::Bark => bark.clone(),
            RootResource::Wood => wood.clone(),
        };
        let location = random_location(LEVEL_MIN, LEVEL_MAX);
        //Cloning everything multiple times is SUPER optimized
        spawn_root(&mut commands, &cube_mesh, &cloned_material, i, &root_resource, Transform::from_translation(location));

        root_around(0.5, location, &mut commands, &cube_mesh, &cloned_material, i, &root_resource);
    }
}

fn root_around(root_chance: f32, location: Vec3, commands: &mut Commands, cube_mesh: &Handle<Mesh>, cloned_material: &Handle<StandardMaterial>, i: i64, root_resource: &RootResource) {
    for x in -1..1 {
        for z in -1..1 {
            if x == 0 && z == 0 {
                return;
            }
            if generate_random_number() > root_chance {
                let next = location + Vec3::new(x as f32, 0.0, z as f32);
                spawn_root(commands, cube_mesh, &cloned_material, i, &root_resource, Transform::from_translation(next));
                root_around(root_chance + 0.1, location, commands, cube_mesh, &cloned_material, i, &root_resource)
            }
        }
    }
}


fn spawn_root(commands: &mut Commands,
    cube_mesh: &Handle<Mesh>,
    cloned_material: &Handle<StandardMaterial>,
    i: i64,
    root_resource: &RootResource,
    transform: Transform) {
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: cloned_material.clone(),
            transform,
            ..default()
        },
        Root { id: i, resource: root_resource.clone()},
    ));
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

fn camera_system(keyboard_input: Res<Input<KeyCode>>, time: Res<Time>, mut query: Query<(&mut Transform), With<Camera>>){
    for (mut transform) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Z) {
            transform.translation.y -= 5.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::X) {
            transform.translation.y += 5.0 * time.delta_seconds();
        }
    }
}

fn movement_system(keyboard_input: Res<Input<KeyCode>>, time: Res<Time>, mut query: Query<(&mut Transform, &Movement), With<Player>>) {
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
