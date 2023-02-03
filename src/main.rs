mod shaders;
use core::f32;


use bevy::{prelude::*};
use rand::{prelude::*, distributions::uniform::SampleUniform};

const LEVEL_MIN: f32 = -300.0;
const LEVEL_MAX: f32 = 300.0;
const CAMERA_OFF_SET: Vec3 = Vec3::new(-2.0, 0.0, 5.0);

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
    resource: RootResource,
    mineable: i32
}



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<shaders::CustomMaterial>::default())
        //.add_plugin(bevy_editor_pls::EditorPlugin)
        //.add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        //.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .insert_resource(ClearColor(Color::GREEN))
        .add_startup_system(setup)
        .add_system(movement_system)
        .add_system(camera_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<shaders::CustomMaterial>>,
) {
    let mut rng = rand::thread_rng();
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0) + CAMERA_OFF_SET).looking_at(Vec3::ZERO, Vec3::Y),
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

    let sap = materials.add(Color::GREEN.into());
    let bark = materials.add(Color::CRIMSON.into());
    let wood = materials.add(Color::BEIGE.into());

    for i in 0..1000 {
        let root_resource = random_resource(&mut rng);

        let cloned_material = match root_resource {
            RootResource::Sap => sap.clone(),
            RootResource::Bark => bark.clone(),
            RootResource::Wood => wood.clone(),
        };
        let location = random_location(&mut rng, LEVEL_MIN, LEVEL_MAX);
        let mut takenVecs = Vec::new();

        //Cloning everything multiple times is SUPER optimized
        spawn_root(&mut commands, &cube_mesh, &cloned_material, i, &root_resource, Transform::from_translation(location), &mut rng, &mut takenVecs);

        root_around(0.3, 0.05, location, &mut commands, &cube_mesh, &cloned_material, i, &root_resource, &mut rng, &mut takenVecs);
    }
}

fn root_around(
    root_chance: f32,
    root_growth: f32,
    location: Vec3,
    commands: &mut Commands,
    cube_mesh: &Handle<Mesh>,
    cloned_material: &Handle<StandardMaterial>,
    i: i64,
    root_resource: &RootResource,
    rng: &mut ThreadRng,
    roots: &mut Vec<Vec3>
) {
    for x in -1..=1 {
        for z in -1..=1 {
            if x == 0 && z == 0 {
                return;
            }
            if generate_random_number(rng) > root_chance {
                let next = location + Vec3::new(x as f32, 0.0, z as f32);
                for vec in roots.iter()  {
                    if vec.x == next.x && vec.z == next.z {
                        return;
                    }
                }
                spawn_root(commands, cube_mesh, &cloned_material, i, &root_resource, Transform::from_translation(next), rng, roots);
                root_around(root_chance + root_growth, root_growth, next, commands, cube_mesh, &cloned_material, i, &root_resource, rng, roots)
            }
        }
    }
}


fn spawn_root(commands: &mut Commands,
    cube_mesh: &Handle<Mesh>,
    cloned_material: &Handle<StandardMaterial>,
    i: i64,
    root_resource: &RootResource,
    transform: Transform,
    rng: &mut ThreadRng,
    roots: &mut Vec<Vec3>) {
    let id = commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: cloned_material.clone(),
            transform,
            ..default()
        },
        Root {
            id: i,
            resource: root_resource.clone(),
            mineable: generate_random_between(rng, 1, 8)},
    )).id();
    roots.push(transform.translation);
}

fn random_location(rng: &mut ThreadRng, min: f32, max: f32) -> Vec3 {
    Vec3 { x: (generate_random_between(rng, min, max)), y: 0.0, z: (generate_random_between(rng, min, max)) }
}

fn random_resource(rng: &mut ThreadRng) -> RootResource {
    let rng = generate_random_number(rng);
    if rng > 0.8 {
        return RootResource::Sap;
    }
    if rng > 0.5 {
        return RootResource::Bark;
    }
    RootResource::Wood
}

fn generate_random_between<T> (rng: &mut ThreadRng, min: T, max: T) -> T
where T: SampleUniform + std::cmp::PartialOrd {
    let range = min..=max;
    rng.gen_range(range)
}

fn generate_random_number(rng: &mut ThreadRng, ) -> f32 {
    rng.gen::<f32>()
}

fn camera_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>) {
    let mut center = Vec3::default();
    let mut count: i32 = 0;
    for (tranform) in player_query.iter()  {
        center += tranform.translation;
        count += 1;
    }
    center = Vec3::new(center.x / count as f32, 0.0, center.z / count as f32);


    
    for (mut transform) in query.iter_mut() {
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
