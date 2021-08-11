// Based on: https://gamedevelopment.tutsplus.com/tutorials/basic-2d-platformer-physics-part-2--cms-25922

use bevy::{core::FixedTimestep, prelude::*};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Collider, RigidBody},
    map::Map,
};

const EPSILON: f32 = 1e-8;

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

fn vec3_to_vec2(v3: Vec3) -> Vec2 {
    Vec2::new(v3.x, v3.y)
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.))
                .with_system(detect_collisions.system().label("detect collisions"))
                .with_system(
                    apply_movements
                        .system()
                        .label("apply movements")
                        .after("detect collisions"),
                ),
        );
    }
}

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

impl<'a> Default for Sweep<'a> {
    fn default() -> Self {
        Self {
            hit: None,
            pos: Vec2::ZERO,
            time: 1.,
        }
    }
}

// AABB implementation
impl Collider {
    /// detect an intersection with point
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

    /// detect an intersection with segment
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

    /// detect an intersection with a static collider
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

    /// detect an intersection with a dynamic collider
    pub fn sweep<'a>(&'a self, other: &'a Collider, delta: Vec2) -> Sweep<'a> {
        let mut sweep = Sweep::default();

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

fn detect_collisions(
    collider_query: Query<&Collider>,
    mut rb_query: Query<(&mut RigidBody, &Collider)>,
    map: Res<Map>,
    time: Res<Time>,
) {
    // first we need to compile a list of changes (compute all the movements and collisions)
    // then we will apply them
    for (mut rigidbody, rb_collider) in rb_query.iter_mut() {
        rigidbody.old_position = rigidbody.position;
        rigidbody.position.x += rigidbody.speed.x * time.delta_seconds();
        rigidbody.position.y += rigidbody.speed.y * time.delta_seconds();

        let delta = vec3_to_vec2(rigidbody.position - rigidbody.old_position);
        let mut nearest = Sweep::default();
        nearest.pos = rb_collider.center + delta;
        for collider in collider_query.iter() {
            let sweep = rb_collider.sweep(collider, delta);
            if sweep.time < nearest.time {
                nearest = sweep;
            }
        }

        if let Some(hit) = nearest.hit {
            println!(
                "Collision! @ {:?} into {:?} -> {:?}",
                rigidbody.position, hit.pos, hit.delta
            );
            rigidbody.position.x += hit.delta.x;
            rigidbody.position.y += hit.delta.y;
        }

        // rigidbody.position.x += nearest.pos.x;
        // rigidbody.position.y += nearest.pos.y;
    }
}

fn apply_movements(mut query: Query<(&mut Transform, &RigidBody, &mut Collider)>) {
    for (mut transform, rigidbody, mut collider) in query.iter_mut() {
        transform.translation = Vec3::new(
            rigidbody.position.x.ceil(),
            rigidbody.position.y.ceil(),
            rigidbody.position.z.ceil(),
        );
        transform.scale = Vec3::new(rigidbody.scale.x, rigidbody.scale.y, rigidbody.scale.z);

        collider.center.x = rigidbody.position.x + collider.half.x;
        collider.center.y = rigidbody.position.y + collider.half.y;
    }
}
