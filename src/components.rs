use bevy::{
    math::{Vec2, Vec3},
    prelude::Transform,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn vec2(&self) -> Vec2 {
        match self {
            Direction::Left => Vec2::new(-1., 0.),
            Direction::Right => Vec2::new(1., 0.),
            Direction::Up => Vec2::new(0., 1.),
            Direction::Down => Vec2::new(0., -1.),
        }
    }
}

pub struct Player {
    pub facing: Direction,
    pub state: PlayerState,

    pub min_jump_speed: f32,
    pub jump_speed: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            facing: Direction::Right,
            state: PlayerState::Stand,

            min_jump_speed: 200.0,
            jump_speed: 410.0,
        }
    }
}

pub enum PlayerState {
    Stand,
    Walk,
    Jump,
}

pub struct PlayerReadyAttack(pub bool);

pub struct RigidBody {
    pub old_position: Vec3,
    pub position: Vec3,

    pub old_speed: Vec3,
    pub speed: Vec3,

    pub scale: Vec3,

    pub was_on_ground: bool,
    pub on_ground: bool,

    pub at_ceiling: bool,
    pub was_at_ceiling: bool,

    pub on_platform: bool,

    pub pushes_right_tile: bool,
    pub pushed_right_tile: bool,
    pub pushes_left_tile: bool,
    pub pushed_left_tile: bool,
    // pub aabb_offset: Vec2,
}
impl Default for RigidBody {
    fn default() -> Self {
        Self {
            old_position: Vec3::ZERO,
            position: Vec3::ZERO,
            old_speed: Vec3::ZERO,
            speed: Vec3::ZERO,
            scale: Vec3::ZERO,
            on_ground: false,
            was_on_ground: false,
            at_ceiling: false,
            was_at_ceiling: false,
            on_platform: false,
            pushes_right_tile: false,
            pushed_right_tile: false,
            pushes_left_tile: false,
            pushed_left_tile: false,
            // aabb_offset: Vec2::ZERO,
        }
    }
}
impl RigidBody {
    pub fn from_transform(transform: Transform) -> Self {
        Self {
            old_position: transform.translation,
            position: transform.translation,
            scale: transform.scale,
            ..Default::default()
        }
    }
}
pub struct Collider {
    pub center: Vec2,
    pub half: Vec2,
}
impl Collider {
    pub fn new(center: Vec2, half: Vec2) -> Self {
        Self { center, half }
    }
}

pub struct Speed(pub Vec3);
impl Default for Speed {
    fn default() -> Self {
        Self(Vec3::new(160.0, 160.0, 1.))
    }
}
impl Speed {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec3::new(x, y, 1.))
    }
}

pub struct Projectile {
    pub direction: Direction,
}

pub struct Tile;
