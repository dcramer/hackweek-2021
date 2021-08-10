use bevy::prelude::*;

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

pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(30.0)
    }
}

pub struct CanJump;
pub struct Jumping {
    pub height: f32,
}

pub struct Projectile {
    pub direction: Direction,
}

pub struct Tile;
