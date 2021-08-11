use std::cmp;

use bevy::sprite::collide_aabb::collide;
use bevy::{core::FixedTimestep, prelude::*};

use crate::components::AABB;
use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Direction, Movable},
    map::Map,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.))
                .with_system(physics.system()),
        );
    }
}

fn physics(mut query: Query<(&mut Transform, &mut Movable)>, map: Res<Map>, time: Res<Time>) {
    for (mut transform, mut movable) in query.iter_mut() {
        movable.old_position = movable.position;
        movable.old_speed = movable.speed;
        movable.was_on_ground = movable.on_ground;
        movable.was_at_ceiling = movable.at_ceiling;
        movable.pushed_right_tile = movable.pushes_right_tile;
        movable.pushed_left_tile = movable.pushes_left_tile;

        movable.position.x += movable.speed.x * time.delta_seconds();
        movable.position.y += movable.speed.y * time.delta_seconds();

        let mut on_platform = false;
        let mut ground_y = 0.;
        if movable.speed.y <= 0.0
            && has_ground(
                &movable.aabb,
                &movable.aabb_offset,
                &map,
                &movable.old_position,
                &movable.position,
                &movable.speed,
                &mut ground_y,
                &mut on_platform,
            )
        {
            // if movable.position.y <= 0.0 {
            // movable.position.y = 0.0;
            movable.speed.y = 0.0;
            movable.position.y = ground_y + movable.aabb.half_size.y - movable.aabb_offset.y;
            movable.on_ground = true;
        } else {
            movable.on_ground = false;
        }
        movable.on_platform = on_platform;
        movable.aabb.center.x = movable.position.x + movable.aabb_offset.x;
        movable.aabb.center.y = movable.position.y + movable.aabb_offset.y;

        transform.translation = Vec3::new(
            movable.position.x.ceil(),
            movable.position.y.ceil(),
            movable.position.z.ceil(),
        );
        transform.scale = Vec3::new(movable.scale.x, movable.scale.y, movable.scale.z);
    }
}

// Based on: https://gamedevelopment.tutsplus.com/tutorials/basic-2d-platformer-physics-part-2--cms-25922
pub fn has_ground(
    aabb: &AABB,
    aabb_offset: &Vec2,
    map: &Res<Map>,
    old_position: &Vec3,
    position: &Vec3,
    speed: &Vec3,
    ground_y: &mut f32,
    on_platform: &mut bool,
) -> bool {
    // find the center of our current object
    let old_center = Vec2::new(
        old_position.x + aabb_offset.x,
        old_position.y + aabb_offset.y,
    );
    let center = Vec2::new(position.x + aabb_offset.x, position.y + aabb_offset.y);
    // find the bottom left sensor
    let old_bottom_left = Vec2::new(
        old_center.x - aabb.half_size.x + Direction::Down.vec2().x + Direction::Left.vec2().x,
        old_center.y - aabb.half_size.y + Direction::Down.vec2().y + Direction::Left.vec2().y,
    );
    let new_bottom_left = Vec2::new(
        center.x - aabb.half_size.x + Direction::Down.vec2().x + Direction::Left.vec2().x,
        center.y - aabb.half_size.y + Direction::Down.vec2().y + Direction::Left.vec2().y,
    );
    // find the bottom right sensor
    // let new_bottom_right = Vec2::new(
    //     new_bottom_left.x + aabb.half_size.x * 2. - 2.,
    //     new_bottom_left.y,
    // );

    let end_y = map.tile_y_at_point(new_bottom_left.y);
    let beg_y = cmp::max(map.tile_y_at_point(old_bottom_left.y), end_y);
    let dist = cmp::max((end_y - beg_y).abs(), 1);

    let mut tile_x: i32;
    let mut tile_y = beg_y;
    while tile_y >= end_y {
        let bottom_left = Vec2::lerp(
            new_bottom_left,
            old_bottom_left,
            ((end_y - tile_y).abs() / dist) as f32,
        );
        let bottom_right = Vec2::new(bottom_left.x + aabb.half_size.x * 2. - 2., bottom_left.y);

        let mut checked_tile_x = bottom_left.x as i32;
        // check tiles "below" us (in case we're overlapping two tiles)
        while checked_tile_x < bottom_right.x as i32 + map.tile_size {
            tile_x = map.tile_x_at_point(checked_tile_x as f32);
            tile_y = map.tile_y_at_point(bottom_left.y);

            *ground_y = (tile_y * map.tile_size) as f32 + map.tile_size as f32 + map.position.y;

            if map.is_obstacle(tile_x, tile_y) {
                *on_platform = false;
                return true;
            } else if map.is_platform(tile_x, tile_y)
                && (bottom_left.y - *ground_y).abs()
                    <= PLATFORM_THRESHOLD + old_position.y - position.y
            {
                *on_platform = true;
            }

            if checked_tile_x as f32 >= bottom_right.x {
                if *on_platform {
                    return true;
                }
                break;
            }

            checked_tile_x += map.tile_size;
        }

        tile_y -= 1;
    }

    false
}
