use bevy::prelude::*;

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

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    state: PlayerState,
    ready_attack: PlayerReadyAttack,
    speed: Speed,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player::default(),
            state: PlayerState::Stand,
            ready_attack: PlayerReadyAttack(true),
            speed: Speed::new(240.0, 0.0),
        }
    }
}

pub struct RigidBody {
    pub old_position: Vec3,
    pub position: Vec3,

    pub old_speed: Vec3,
    pub speed: Vec3,

    pub scale: Vec3,

    pub on_platform: bool,
    pub on_ground: bool,
    pub at_ceiling: bool,
    pub at_right_tile: bool,
    pub at_left_tile: bool,
}
impl Default for RigidBody {
    fn default() -> Self {
        Self {
            old_position: Vec3::ZERO,
            position: Vec3::ZERO,
            old_speed: Vec3::ZERO,
            speed: Vec3::ZERO,
            scale: Vec3::ZERO,
            on_ground: true,
            at_ceiling: false,
            on_platform: false,
            at_right_tile: false,
            at_left_tile: false,
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

#[derive(Debug, PartialEq)]
pub struct Collider {
    pub center: Vec2,
    pub half: Vec2,

    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}
impl Collider {
    pub fn new(center: Vec2, half: Vec2) -> Self {
        Self {
            center,
            half,
            ..Default::default()
        }
    }

    pub fn from_position(pos: Vec3, half: Vec2) -> Self {
        Self::new(Vec2::new(pos.x + half.x, pos.y + half.y), half)
    }

    pub fn edges(mut self, top: bool, right: bool, bottom: bool, left: bool) -> Self {
        self.top = top;
        self.right = right;
        self.bottom = bottom;
        self.left = left;
        self
    }

    pub fn center_from(&mut self, pos: Vec3) {
        self.center.x = pos.x + self.half.x;
        self.center.y = pos.y + self.half.y;
    }
}
impl Default for Collider {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            half: Vec2::ZERO,
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
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
