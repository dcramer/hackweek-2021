use bevy::math::Vec2;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct Player {
    pub facing: Direction,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            facing: Direction::Right,
        }
    }
}

pub struct PlayerReadyAttack(pub bool);

#[derive(Copy, Clone, PartialEq)]
pub struct Velocity(pub f32);

#[derive(Copy, Clone, PartialEq)]
pub struct Gravity(pub f32);

pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500.0)
    }
}

pub struct Projectile {
    pub direction: Direction,
}

pub struct Tile;

pub struct Collider {
    pub size: Vec2,
}

pub struct Moving;

impl Collider {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: Vec2::new(width, height),
        }
    }
}
