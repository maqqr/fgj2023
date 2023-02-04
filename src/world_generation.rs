use bevy::prelude::*;
use rand::rngs::ThreadRng;
use crate::{*, vec3i::Vec3i, shaders::CustomMaterial, utils::*};

pub struct WorldGenerator<'a> {
    pub cube_mesh: &'a Handle<Mesh>,
    pub plane_mesh: &'a Handle<Mesh>,
    pub material_map: &'a HashMap<RootResource, Handle<CustomMaterial>>,
    pub ground_material: &'a Handle<CustomMaterial>,
    pub rng: &'a mut ThreadRng,
    pub blockmap: &'a mut BlockMap,
}

impl WorldGenerator<'_> {
    pub fn root_around(
        &mut self,
        i: i64,
        location: &Vec3i,
        root_resource: RootResource,
        root_chance: f32,
        root_growth: f32,
        commands: &mut Commands,
    ) {
        for x in -1..=1 {
            for z in -1..=1 {
                if x == 0 && z == 0 {
                    return;
                }
                if generate_random_number(self.rng) > root_chance {
                    let next = *location + Vec3i::new(x, 0, z);
                    if self.blockmap.entities.contains_key(&next) {
                        return;
                    }
                    self.spawn_root_block(i, &next, root_resource, commands);
                    self.root_around(i, &next, root_resource, root_chance + root_growth, root_growth, commands);
                }
            }
        }
    }

    pub fn make_trunk(&mut self, i: i64, position: &Vec3i, root_resource: RootResource, commands: &mut Commands) {
        // TODO: This trunk needs some thickness
        for y in 0..12 {
            self.spawn_root_block(i, &(*position + (0, y, 0).into()), root_resource, commands);
        }
    }

    pub fn spawn_root_block(&mut self, i: i64, position: &Vec3i, root_resource: RootResource, commands: &mut Commands) {
        if self.blockmap.entities.contains_key(position) {
            return;
        }
        let material = self.material_map.get(&root_resource).unwrap(); // this will crash if material is not found
        let block = self.spawn_block(position, material, commands);
        commands
            .entity(block)
            .insert(Root {
                id: i,
                resource: root_resource,
                mineable: generate_random_between(self.rng, 1, 8),
            })
            .insert(Collider::cuboid(0.5, 0.5, 0.5));
    }

    pub fn spawn_block(&mut self, position: &Vec3i, material: &Handle<CustomMaterial>, commands: &mut Commands) -> Entity {
        let entity = commands
            .spawn((
                MaterialMeshBundle {
                    mesh: self.cube_mesh.clone(),
                    material: material.clone(),
                    transform: (*position).into(),
                    ..default()
                },
                BlockPosition(*position),
            ))
            .id();
        self.blockmap.entities.insert(*position, entity);
        entity
    }

    pub fn make_ground_plane(&mut self, commands: &mut Commands) {
        for x in LEVEL_MIN as i64..LEVEL_MAX as i64 {
            for z in LEVEL_MIN as i64..LEVEL_MAX as i64 {
                // Ground block is simplified into a plane mesh
                commands.spawn((MaterialMeshBundle {
                    mesh: self.plane_mesh.clone(),
                    material: self.ground_material.clone(),
                    transform: Transform::from_translation(Vec3::new(x as f32, -0.5, z as f32)),
                    ..default()
                },));
            }
        }

        // Add one large collider for all ground blocks
        commands.spawn((
            TransformBundle::from(Transform::from_xyz(0.0, -1.0, 0.0)),
            Collider::cuboid(LEVEL_MAX, 0.5, LEVEL_MAX),
        ));
    }
}
