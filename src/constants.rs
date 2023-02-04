use bevy::{prelude::{KeyCode, Vec3}, utils::HashMap};

pub const LEVEL_MIN: f32 = -100.0;
pub const LEVEL_MAX: f32 = 100.0;

pub const INITIAL_CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 11.0, 15.0);

pub const DEFAULT_BENDING: f32 = 0.015;

// lazy_static is used because HashMap cannot be created at compile time
lazy_static!{
    pub static ref KEYS: HashMap<KeyCode, Vec3> = [
        (KeyCode::Up, (0.0, 0.0, -1.0).into()),
        (KeyCode::Down, (0.0, 0.0, 1.0).into()),
        (KeyCode::Left, (-1.0, 0.0, 0.0).into()),
        (KeyCode::Right, (1.0, 0.0, 0.0).into()),
    ].iter().copied().collect();
}
