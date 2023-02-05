use std::f32::consts::PI;

use bevy::{prelude::{KeyCode, Vec3}, utils::HashMap};

pub const FIELD_OF_VIEW: f32 = 80.0 * (PI / 180.0);

pub const LEVEL_MIN: f32 = -80.0;
pub const LEVEL_MAX: f32 = 80.0;

pub const INITIAL_CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 4.0, 4.0);

pub const DEFAULT_BENDING: f32 = 0.03;

// lazy_static is used because HashMap cannot be created at compile time
lazy_static!{
    pub static ref KEYS: HashMap<KeyCode, Vec3> = [
        (KeyCode::Up, (0.0, 0.0, -1.0).into()),
        (KeyCode::Down, (0.0, 0.0, 1.0).into()),
        (KeyCode::Left, (-1.0, 0.0, 0.0).into()),
        (KeyCode::Right, (1.0, 0.0, 0.0).into()),
        (KeyCode::W, (0.0, 0.0, -1.0).into()),
        (KeyCode::S, (0.0, 0.0, 1.0).into()),
        (KeyCode::A, (-1.0, 0.0, 0.0).into()),
        (KeyCode::D, (1.0, 0.0, 0.0).into()),
    ].iter().copied().collect();
}
