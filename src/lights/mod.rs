use bevy::prelude::*;
pub mod node;

/// A point light
#[derive(Debug, Reflect)]
#[reflect(Component)]
pub struct PointLight {
    pub color: Color,
}

impl Default for PointLight {
    fn default() -> Self {
        PointLight {
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }
}

pub struct SunLight {
    pub color: Color,
    pub direction: Vec3,
}

pub struct AmbientLight {
    pub color: Color,
}
