// Based on: https://gamedevelopment.tutsplus.com/tutorials/basic-2d-platformer-physics-part-2--cms-25922
use std::cmp;

use bevy::{core::FixedTimestep, prelude::*};

use crate::components::Collider;
use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Direction, RigidBody},
    map::Map,
};

pub struct PhysicsPlugin;

pub struct ColliderMaterials {
    collider_border: Handle<ColorMaterial>,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .add_startup_system(setup_debugger.system())
            // .add_system(debug_colliders.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0 / 60.))
                    .with_system(physics.system()),
            );
    }
}

fn setup_debugger(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(ColliderMaterials {
        collider_border: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
    });
}

fn round_vector(v: Vec2) -> Vec2 {
    Vec2::new(v.x.round(), v.y.round())
}

fn add_vectors2(one: Vec2, two: Vec2) -> Vec2 {
    Vec2::new(one.x + two.x, one.y + two.y)
}

fn add_vectors3(one: Vec3, two: Vec3) -> Vec3 {
    Vec3::new(one.x + two.x, one.y + two.y, one.z + two.z / 2.)
}

fn add_vectors23(one: Vec2, two: Vec3) -> Vec2 {
    Vec2::new(one.x + two.x, one.y + two.y)
}

fn minf(a: f32, b: f32) -> f32 {
    if a > b {
        b
    } else {
        a
    }
}

fn maxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub struct ColliderDebugger;

pub struct Hit<'a> {
    collider: &'a Collider,
    pos: Vec2,
    delta: Vec2,
    normal: Vec2,
    time: f32,
}
impl<'a> Hit<'a> {
    pub fn new(collider: &'a Collider) -> Self {
        Self {
            collider,
            pos: Vec2::ZERO,
            delta: Vec2::ZERO,
            normal: Vec2::ZERO,
            time: 1.,
        }
    }
}

pub struct Sweep<'a> {
    hit: Option<Hit<'a>>,
    pos: Vec2,
    time: f32,
}

pub const EPSILON: f32 = 1e-8;

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn sign(value: f32) -> f32 {
    if value < 0. {
        -1.
    } else {
        1.
    }
}

// AABB implementation
impl Collider {
    pub fn intersect_point<'a>(&'a self, pos: Vec2) -> Option<Hit<'a>> {
        let dx = pos.x - self.center.x;
        let px = self.half.x - dx.abs();
        if px <= 0. {
            return None;
        }

        let dy = pos.y - self.center.y;
        let py = self.half.y - dy.abs();
        if py <= 0. {
            return None;
        }

        let mut hit = Hit::new(self);
        if px < py {
            let sx = sign(dx);
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.center.x + (self.half.x * sx);
            hit.pos.y = pos.y;
        } else {
            let sy = sign(dy);
            hit.delta.y = py * sy;
            hit.normal.y = sy;
            hit.pos.x = pos.x;
            hit.pos.y = self.center.y + (self.half.y * sy);
        }
        Some(hit)
    }

    pub fn intersect_segment<'a>(
        &'a self,
        pos: Vec2,
        delta: Vec2,
        padding_x: f32,
        padding_y: f32,
    ) -> Option<Hit<'a>> {
        let scale_x = 1. / delta.x;
        let scale_y = 1. / delta.y;
        let sign_x = sign(scale_x);
        let sign_y = sign(scale_y);
        let near_time_x = (self.center.x - sign_x * (self.half.x + padding_x) - pos.x) * scale_x;
        let near_time_y = (self.center.y - sign_y * (self.half.y + padding_y) - pos.y) * scale_y;
        let far_time_x = (self.center.x + sign_x * (self.half.x + padding_x) - pos.x) * scale_x;
        let far_time_y = (self.center.y + sign_y * (self.half.y + padding_y) - pos.y) * scale_y;

        if near_time_x > far_time_y || near_time_y > far_time_x {
            return None;
        }

        let near_time = if near_time_x > near_time_y {
            near_time_x
        } else {
            near_time_y
        };
        let far_time = if far_time_x < far_time_y {
            far_time_x
        } else {
            far_time_y
        };

        if near_time >= 1. || far_time <= 0. {
            return None;
        }

        let mut hit = Hit::new(self);
        hit.time = clamp(near_time, 0., 1.);
        if near_time_x > near_time_y {
            hit.normal.x = -sign_x;
            hit.normal.y = 0.;
        } else {
            hit.normal.x = 0.;
            hit.normal.y = -sign_y;
        }
        hit.delta.x = (1. - hit.time) * -delta.x;
        hit.delta.y = (1.0 - hit.time) * -delta.y;
        hit.pos.x = pos.x + delta.x * hit.time;
        hit.pos.y = pos.y + delta.y * hit.time;
        Some(hit)
    }

    fn intersect<'a>(&'a self, other: &'a Collider) -> Option<Hit<'a>> {
        let dx = other.center.x - self.center.x;
        let px = (other.half.x + self.half.x) - dx.abs();
        if px <= 0. {
            return None;
        }

        let dy = other.center.y - self.center.y;
        let py = (other.half.y + self.center.y) - dy.abs();
        if py <= 0. {
            return None;
        }

        let mut hit = Hit::new(&self);
        if px < py {
            let sx = sign(dx);
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.center.x + (self.half.x * sx);
            hit.pos.y = other.center.y;
        } else {
            let sy = sign(dy);
            hit.delta.y = py * sy;
            hit.normal.y = sy;
            hit.pos.x = other.center.x;
            hit.pos.y = self.center.y + (self.half.y * sy);
        }
        Some(hit)
    }

    pub fn sweep<'a>(&'a self, other: &'a Collider, delta: Vec2) -> Sweep<'a> {
        let mut sweep = Sweep {
            hit: None,
            pos: Vec2::ZERO,
            time: 1.,
        };

        if delta.x == 0. && delta.y == 0. {
            sweep.pos.x = other.center.x;
            sweep.pos.y = other.center.y;
            if let Some(mut hit) = self.intersect(other) {
                hit.time = 0.;
                sweep.time = hit.time;
                sweep.hit = Some(hit);
            } else {
                sweep.time = 1.;
            }
            return sweep;
        }

        if let Some(mut hit) =
            self.intersect_segment(other.center, delta, other.half.x, other.half.y)
        {
            sweep.time = clamp(hit.time - EPSILON, 0., 1.);
            sweep.pos.x = other.center.x + delta.x * sweep.time;
            sweep.pos.y = other.center.y + delta.y * sweep.time;
            let direction = delta.clone();
            direction.normalize();
            hit.pos.x = clamp(
                hit.pos.x + direction.x * other.half.x,
                self.center.x - self.half.x,
                self.center.x + self.half.x,
            );
            hit.pos.y = clamp(
                hit.pos.y + direction.y * other.half.y,
                self.center.y - self.half.y,
                self.center.y + self.half.y,
            );
            sweep.hit = Some(hit);
        } else {
            sweep.pos.x = other.center.x + delta.x;
            sweep.pos.y = other.center.y + delta.y;
            sweep.time = 1.;
        }
        sweep
    }
}

fn debug_colliders(
    mut commands: Commands,
    materials: Res<ColliderMaterials>,
    mut query: Query<(Entity, &RigidBody, &Collider, Without<ColliderDebugger>)>,
) {
    for (entity, rigidbody, collider, _) in query.iter_mut() {
        commands
            .entity(entity)
            .with_children(|parent| {
                // top
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    global_transform: GlobalTransform::from_translation(Vec3::new(
                        collider.center.x - collider.half.x,
                        collider.center.y + collider.half.y,
                        500.,
                    )),
                    sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
                    ..Default::default()
                });
                // bottom
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    global_transform: GlobalTransform::from_translation(Vec3::new(
                        collider.center.x - collider.half.x,
                        collider.center.y - collider.half.y,
                        500.,
                    )),
                    sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
                    ..Default::default()
                });
                // left
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    global_transform: GlobalTransform::from_translation(Vec3::new(
                        collider.center.x - collider.half.x,
                        collider.center.y - collider.half.y,
                        500.,
                    )),
                    sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
                    ..Default::default()
                });
                // right
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    global_transform: GlobalTransform::from_translation(Vec3::new(
                        collider.center.x + collider.half.x,
                        collider.center.y - collider.half.y,
                        500.,
                    )),
                    sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
                    ..Default::default()
                });
            })
            .insert(ColliderDebugger);
    }
}

fn physics(
    mut query: Query<(&mut Transform, &mut RigidBody, &mut Collider)>,
    map: Res<Map>,
    time: Res<Time>,
) {
    for (mut transform, mut rigidbody, mut collider) in query.iter_mut() {
        rigidbody.old_position = rigidbody.position;
        rigidbody.old_speed = rigidbody.speed;
        rigidbody.was_on_ground = rigidbody.on_ground;
        rigidbody.was_at_ceiling = rigidbody.at_ceiling;
        rigidbody.pushed_right_tile = rigidbody.pushes_right_tile;
        rigidbody.pushed_left_tile = rigidbody.pushes_left_tile;

        rigidbody.position.x += rigidbody.speed.x * time.delta_seconds();
        rigidbody.position.y += rigidbody.speed.y * time.delta_seconds();

        let mut left_tile_x = 0.;
        if rigidbody.speed.x <= 0.0
            && collides_with_left_tile(
                &map,
                collider.half,
                rigidbody.old_position,
                rigidbody.position,
                &mut left_tile_x,
            )
        {
            if rigidbody.old_position.x - collider.half.x >= left_tile_x {
                rigidbody.position.x = left_tile_x;
                rigidbody.pushes_left_tile = true;
            }
            if rigidbody.speed.x < 0. {
                rigidbody.speed.x = 0.0;
            }
        } else {
            rigidbody.pushes_left_tile = false;
        }

        let mut right_tile_x = 0.;
        if rigidbody.speed.x >= 0.0
            && collides_with_right_tile(
                &map,
                collider.half,
                rigidbody.old_position,
                rigidbody.position,
                &mut right_tile_x,
            )
        {
            if rigidbody.old_position.x + collider.half.x <= right_tile_x {
                rigidbody.position.x = right_tile_x;
                rigidbody.pushes_right_tile = true;
            }
            if rigidbody.speed.x > 0. {
                rigidbody.speed.x = 0.0;
            }
        } else {
            rigidbody.pushes_right_tile = false;
        }

        let mut on_platform = false;
        let mut ground_y = 0.;
        if rigidbody.speed.y <= 0.0
            && has_ground(
                &map,
                collider.half,
                rigidbody.old_position,
                rigidbody.position,
                &mut ground_y,
                &mut on_platform,
            )
        {
            rigidbody.speed.y = 0.0;
            rigidbody.position.y = ground_y;
            rigidbody.on_ground = true;
        } else {
            rigidbody.on_ground = false;
        }
        rigidbody.on_platform = on_platform;

        // let mut ceiling_y = 0.;
        // if rigidbody.speed.y >= 0.0
        //     && has_ceiling(
        //         &map,
        //         collider.half,
        //         collider_offset,
        //         rigidbody.old_position,
        //         rigidbody.position,
        //         &mut ceiling_y,
        //     )
        // {
        //     rigidbody.position.y = ceiling_y - 1.;
        //     rigidbody.speed.y = 0.0;
        //     rigidbody.at_ceiling = true;
        // } else {
        //     rigidbody.at_ceiling = false;
        // }

        collider.center.x = rigidbody.position.x + collider.half.x;
        collider.center.y = rigidbody.position.y + collider.half.y;

        transform.translation = Vec3::new(
            rigidbody.position.x.ceil(),
            rigidbody.position.y.ceil(),
            rigidbody.position.z.ceil(),
        );
        transform.scale = Vec3::new(rigidbody.scale.x, rigidbody.scale.y, rigidbody.scale.z);
    }
}

pub fn has_ground(
    map: &Map,
    half: Vec2,
    old_position: Vec3,
    position: Vec3,
    ground_y: &mut f32,
    on_platform: &mut bool,
) -> bool {
    // find the center of our current object
    let old_center = add_vectors23(half, old_position);
    let center = add_vectors23(half, position);

    // find the bottom left sensor
    let old_bottom_left = round_vector(Vec2::new(
        old_center.x - half.x - Direction::Up.vec2().x + Direction::Right.vec2().x,
        old_center.y - half.y - Direction::Up.vec2().y + Direction::Right.vec2().y,
    ));
    let new_bottom_left = round_vector(Vec2::new(
        center.x - half.x - Direction::Up.vec2().x + Direction::Right.vec2().x,
        center.y - half.y - Direction::Up.vec2().y + Direction::Right.vec2().y,
    ));

    let end_y = map.tile_y_at_point(new_bottom_left.y);
    let beg_y = cmp::max(map.tile_y_at_point(old_bottom_left.y) - 1, end_y);
    let dist = cmp::max((end_y - beg_y).abs(), 1);

    let mut tile_x: i32;
    let mut tile_y = beg_y;
    while tile_y >= end_y {
        let bottom_left = Vec2::lerp(
            new_bottom_left,
            old_bottom_left,
            ((end_y - tile_y).abs() / dist) as f32,
        );
        let bottom_right = Vec2::new(bottom_left.x + half.x * 2. - 2., bottom_left.y);

        let mut checked_tile_x = bottom_left.x as i32;
        // check tiles "below" us (in case we're overlapping two tiles)
        while checked_tile_x < bottom_right.x as i32 + map.tile_size {
            checked_tile_x = cmp::min(checked_tile_x, bottom_right.x as i32);

            tile_x = map.tile_x_at_point(checked_tile_x as f32);

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

pub fn has_ceiling(
    map: &Map,
    half: Vec2,
    old_position: Vec3,
    position: Vec3,
    ceiling_y: &mut f32,
) -> bool {
    // find the center of our current object
    let old_center = add_vectors23(half, old_position);
    let center = add_vectors23(half, position);

    // find the bottom left sensor
    let old_top_right = round_vector(Vec2::new(
        old_center.x + half.x + Direction::Up.vec2().x + Direction::Right.vec2().x,
        old_center.y + half.y + Direction::Up.vec2().y + Direction::Right.vec2().y,
    ));
    let new_top_right = round_vector(Vec2::new(
        center.x - half.x + Direction::Up.vec2().x + Direction::Right.vec2().x,
        center.y - half.y + Direction::Up.vec2().y + Direction::Right.vec2().y,
    ));

    *ceiling_y = 0.;

    let end_y = map.tile_y_at_point(new_top_right.y);
    let beg_y = cmp::min(map.tile_y_at_point(old_top_right.y) + 1, end_y);
    let dist = cmp::max((end_y - beg_y).abs(), 1);

    let mut tile_x: i32;
    let mut tile_y = beg_y;
    while tile_y <= end_y {
        let top_right = Vec2::lerp(
            new_top_right,
            old_top_right,
            ((end_y - tile_y).abs() / dist) as f32,
        );
        let top_left = Vec2::new(top_right.x - half.x * 2. + 2., top_right.y);

        let mut checked_tile_x = top_left.x as i32;
        // check tiles "below" us (in case we're overlapping two tiles)
        while checked_tile_x < top_right.x as i32 + map.tile_size {
            checked_tile_x = cmp::max(checked_tile_x, top_right.x as i32);

            tile_x = map.tile_x_at_point(checked_tile_x as f32);

            *ceiling_y = (tile_y * map.tile_size) as f32 - half.y - map.position.y;

            if map.is_obstacle(tile_x, tile_y) {
                println!("Collided with {},{}", tile_x, tile_y);
                return true;
            }

            checked_tile_x += map.tile_size;
        }

        tile_y += 1;
    }

    false
}

pub fn collides_with_left_tile(
    map: &Map,
    half: Vec2,
    old_position: Vec3,
    position: Vec3,
    left_tile_x: &mut f32,
) -> bool {
    // find the center of our current object
    let old_center = add_vectors23(half, old_position);
    let center = add_vectors23(half, position);

    // find the bottom left sensor
    let old_bottom_left = round_vector(Vec2::new(
        old_center.x - half.x + Direction::Left.vec2().x,
        old_center.y - half.y + Direction::Left.vec2().y,
    ));
    let new_bottom_left = round_vector(Vec2::new(
        center.x - half.x + Direction::Left.vec2().x,
        center.y - half.y + Direction::Left.vec2().y,
    ));
    // let new_top_left = round_vector(Vec2::new(
    //     new_bottom_left.x,
    //     new_bottom_left.y + aabb.half.y * 2.,
    // ));

    *left_tile_x = 0.;

    let end_x = map.tile_x_at_point(new_bottom_left.x);
    let beg_x = cmp::max(map.tile_x_at_point(old_bottom_left.x) - 1, end_x);
    let dist = cmp::max((end_x - beg_x).abs(), 1);

    let mut tile_x = beg_x;
    let mut tile_y: i32;
    while tile_x >= end_x {
        let bottom_left = Vec2::lerp(
            new_bottom_left,
            old_bottom_left,
            (end_x - tile_x).abs() as f32 / dist as f32,
        );
        let top_left = Vec2::new(bottom_left.x, bottom_left.y + half.y * 2.);

        let mut checked_tile_y = bottom_left.y as i32;
        // check tiles "below" us (in case we're overlapping two tiles)
        while checked_tile_y < top_left.y as i32 + map.tile_size {
            checked_tile_y = cmp::min(checked_tile_y, top_left.y as i32);

            tile_y = map.tile_y_at_point(checked_tile_y as f32);

            if map.is_obstacle(tile_x, tile_y) {
                *left_tile_x =
                    (tile_x * map.tile_size) as f32 + map.tile_size as f32 + map.position.x;
                return true;
            }

            checked_tile_y += map.tile_size;
        }

        tile_x -= 1;
    }

    false
}

pub fn collides_with_right_tile(
    map: &Map,
    half: Vec2,
    old_position: Vec3,
    position: Vec3,
    right_tile_x: &mut f32,
) -> bool {
    // find the center of our current object
    let old_center = add_vectors23(half, old_position);
    let center = add_vectors23(half, position);

    // find the bottom right sensor
    let old_bottom_right = round_vector(Vec2::new(
        old_center.x + half.x + Direction::Right.vec2().x,
        old_center.y - half.y + Direction::Right.vec2().y,
    ));
    let new_bottom_right = round_vector(Vec2::new(
        center.x + half.x + Direction::Right.vec2().x,
        center.y - half.y + Direction::Right.vec2().y,
    ));
    // let new_top_right = round_vector(Vec2::new(
    //     new_bottom_right.x,
    //     new_bottom_right.y + aabb.half.y * 2.,
    // ));

    *right_tile_x = 0.;

    let end_x = map.tile_x_at_point(new_bottom_right.x);
    let beg_x = cmp::min(map.tile_x_at_point(old_bottom_right.x) + 1, end_x);
    let dist = cmp::max((end_x - beg_x).abs(), 1);

    let mut tile_x = beg_x;
    let mut tile_y: i32;
    while tile_x <= end_x {
        let bottom_right = Vec2::lerp(
            new_bottom_right,
            old_bottom_right,
            (end_x - tile_x).abs() as f32 / dist as f32,
        );
        let top_right = Vec2::new(bottom_right.x, bottom_right.y + half.y * 2.);

        let mut checked_tile_y = bottom_right.y as i32;
        // check tiles "below" us (in case we're overlapping two tiles)
        while checked_tile_y < top_right.y as i32 + map.tile_size {
            checked_tile_y = cmp::min(checked_tile_y, top_right.y as i32);

            tile_y = map.tile_y_at_point(checked_tile_y as f32);

            if map.is_obstacle(tile_x, tile_y) {
                *right_tile_x =
                    (tile_x * map.tile_size) as f32 - map.tile_size as f32 + map.position.x;
                return true;
            }

            checked_tile_y += map.tile_size;
        }

        tile_x += 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_prefab() -> Map {
        let prefab: (&str, i32, i32) = (
            "
            -X-#
            -##=
            ^^^^
            ",
            4,
            3,
        );
        let mut map = Map::from_prefab(prefab);
        // move it to 0,0 to make this easier to grok
        map.position = Vec3::ZERO;
        map.tile_size = 4;

        assert_eq!(map.position.y, 0.);
        assert_eq!(map.position.x, 0.);

        map
    }

    #[test]
    fn test_has_ground() {
        let map = build_prefab();

        let aabb_half_size = Vec2::new(2., 2.);
        let aabb_offset = Vec2::new(2., 2.);

        let test_matrix = vec![
            // old, new, right_tile_x, on_platform, result
            // ([0., 0., 0.], [0., 0., 0.], 0., true),

            // round down to airspace
            ([0., 8., 0.], [0., 6., 0.], 0., false, false),
            // immmediate tile below is colliding, but doesnt require movement
            ([0., 8., 0.], [0., 4., 0.], 4., false, true),
            // ([8., 8., 0.], [0., 8., 0.], 0., false, false),
            // ([8., 8., 0.], [0., 4., 0.], 0., false, false),
            // ([8., 8., 0.], [8., 4., 0.], 0., false, true),
            // ([0., 4., 0.], [0., 0., 0.], 0., false, true),
        ];

        let mut ground_y = 0.;
        let mut on_platform: bool = false;
        for (test_idx, (old_pos, new_pos, exp_ground_y, exp_on_platform, result)) in
            test_matrix.iter().enumerate()
        {
            println!("Running collision test {}", test_idx);
            ground_y = 0.;

            let rv = has_ground(
                &map,
                aabb_half_size,
                aabb_offset,
                Vec3::from_slice_unaligned(old_pos),
                Vec3::from_slice_unaligned(new_pos),
                &mut ground_y,
                &mut on_platform,
            );
            assert_eq!(
                rv, *result,
                "unexpected collision result, ground_y={}",
                ground_y
            );
            if rv {
                assert_eq!(ground_y, *exp_ground_y, "ground_y");
                assert_eq!(on_platform, *exp_on_platform, "on_platform");
            }
        }
    }

    #[test]
    fn test_collides_with_right_tile() {
        let map = build_prefab();

        let aabb_half_size = Vec2::new(2., 2.);
        let aabb_offset = Vec2::new(2., 2.);

        let test_matrix = vec![
            // old, new, right_tile_x, result
            // ([0., 0., 0.], [0., 0., 0.], 0., true),
            ([8., 8., 0.], [8., 8., 0.], 0., false),
            ([8., 8., 0.], [12., 8., 0.], 0., false),
            ([4., 4., 0.], [8., 4., 0.], 4., true),
        ];

        let mut right_tile_x = 0.;
        for (test_idx, (old_pos, new_pos, exp_right_tile_x, result)) in
            test_matrix.iter().enumerate()
        {
            println!("Running collision test {}", test_idx);
            right_tile_x = 0.;

            let rv = collides_with_right_tile(
                &map,
                aabb_half_size,
                aabb_offset,
                Vec3::from_slice_unaligned(old_pos),
                Vec3::from_slice_unaligned(new_pos),
                &mut right_tile_x,
            );
            assert_eq!(
                rv, *result,
                "unexpected collision result, right_tile_x={}",
                right_tile_x
            );
            if rv {
                assert_eq!(right_tile_x, *exp_right_tile_x);
            }
        }
    }
}
