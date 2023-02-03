use bevy::prelude::*;
use rand::rngs::ThreadRng;
use crate::{*, vec3i::Vec3i, shaders::CustomMaterial, utils::*};

pub struct RootGenerator<'a> {
    pub i: i64,
    pub cube_mesh: &'a Handle<Mesh>,
    pub cloned_material: &'a Handle<CustomMaterial>,
    pub root_resource: &'a RootResource,
    pub rng: &'a mut ThreadRng,
    pub blockmap: &'a mut BlockMap,
}

impl RootGenerator<'_> {
    pub fn root_around(&mut self,
        location: &Vec3i,
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
                    self.spawn_root(&next, commands);
                    self.root_around(&next, root_chance + root_growth, root_growth, commands);
                }
            }
        }
    }

    pub fn spawn_root(&mut self, position: &Vec3i, commands: &mut Commands) {
        let id = commands.spawn((
            MaterialMeshBundle {
                mesh: self.cube_mesh.clone(),
                material: self.cloned_material.clone(),
                transform: (*position).into(),
                ..default()
            },
            Root {
                id: self.i,
                resource: self.root_resource.clone(),
                mineable: generate_random_between(self.rng, 1, 8)},
        )).id();
        self.blockmap.entities.insert(*position, id);
    }
}