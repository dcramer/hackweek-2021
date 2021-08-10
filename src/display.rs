use bevy::math::Vec2;

use crate::constants::TILE_SIZE;

pub fn normalize_pos(pos: Vec2) -> Vec2 {
    Vec2::new(pos.x * TILE_SIZE, pos.y * TILE_SIZE)
}
