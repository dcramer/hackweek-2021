// based on http://noonat.github.io/intersect/

use bevy::{core::FixedTimestep, prelude::*};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Collider, RigidBody},
    constants::GRAVITY,
};

const EPSILON: f32 = 1e-8;

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
    /// the point of contact between the two objects (or an estimation of it, in some sweep tests).
    pos: Vec2,
    /// the overlap between the two objects, and is a vector that can be added to the colliding object’s position to move it back to a non-colliding state.
    delta: Vec2,
    /// the surface normal at the point of contact.
    normal: Vec2,
    /// defined for segment and sweep intersections, and is a fraction from 0 to 1 indicating how far along the line the collision occurred. (This is the tt value for the line equation L(t) = A + t(B - A)L(t)=A+t(B−A))
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
    /// Hit object if there was a collision, or null if not.
    hit: Option<Hit<'a>>,
    /// the furthest point the object reached along the swept path before it hit something.
    pos: Vec2,
    /// a copy of hit.time, offset by epsilon, or 1.0 if the object didn’t hit anything during the sweep.
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
        hit.time = near_time.clamp(0., 1.);
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
            sweep.time = (hit.time - EPSILON).clamp(0., 1.);
            sweep.pos.x = other.center.x + delta.x * sweep.time;
            sweep.pos.y = other.center.y + delta.y * sweep.time;
            let direction = delta.clone();
            direction.normalize();
            hit.pos.x = (hit.pos.x + direction.x * other.half.x)
                .clamp(self.center.x - self.half.x, self.center.x + self.half.x);
            hit.pos.y = (hit.pos.y + direction.y * other.half.y)
                .clamp(self.center.y - self.half.y, self.center.y + self.half.y);
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
    time: Res<Time>,
) {
    // first we need to compile a list of changes (compute all the movements and collisions)
    // then we will apply them
    for (mut body, rb_collider) in rb_query.iter_mut() {
        body.old_position = body.position;

        body.position.x += body.speed.x * time.delta_seconds();
        body.position.y += body.speed.y * time.delta_seconds();

        // dont compute collisions if we haven't moved
        if body.old_position == body.position {
            continue;
        }

        let old_center = rb_collider.center;
        let new_center = Vec2::new(
            body.position.x + rb_collider.half.x,
            body.position.y + rb_collider.half.y,
        );

        // gross - we mutate the collider later, but now we need an updated object to compute our offsets
        let new_collider = Collider::new(new_center, rb_collider.half);

        let delta = new_center - old_center;
        let mut nearest = Sweep::default();
        nearest.pos = new_center + delta;
        for collider in collider_query.iter().filter(|c| *c != rb_collider) {
            let sweep = new_collider.sweep(collider, delta);
            if sweep.time < nearest.time {
                nearest = sweep;
            }
        }

        body.on_ground = false;
        body.at_ceiling = false;
        body.at_left_tile = false;
        body.at_right_tile = false;

        if let Some(hit) = nearest.hit {
            body.position.x += hit.delta.x;
            body.position.y += hit.delta.y;

            if (hit.delta.x < 0. && body.speed.x > 0.) || (hit.delta.x > 0. && body.speed.x < 0.) {
                info!("collided with side tile {:?} -> {:?}", hit.pos, hit.delta);
                body.speed.x = 0.;
                // body.at_left_tile = hit.delta.x > 0.;
                // body.at_right_tile = hit.delta.x < 0.;
            }

            if hit.delta.y > 0. && body.speed.y < 0. {
                info!("collided with ground {:?} -> {:?}", hit.pos, hit.delta);
                body.speed.y = 0.;
                body.on_ground = true;
            }

            if hit.delta.y < 0. && body.speed.y > 0. {
                info!("collided with ceiling {:?}", hit.pos);
                body.speed.y = GRAVITY * time.delta_seconds();
                body.at_ceiling = true;
            }
        }
    }
}

fn apply_movements(mut query: Query<(&mut Transform, &RigidBody, &mut Collider)>) {
    for (mut transform, body, mut collider) in query.iter_mut() {
        transform.translation = Vec3::new(
            body.position.x.ceil(),
            body.position.y.ceil(),
            body.position.z.ceil(),
        );
        transform.scale = Vec3::new(body.scale.x, body.scale.y, body.scale.z);

        collider.center.x = body.position.x + collider.half.x;
        collider.center.y = body.position.y + collider.half.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect_point_does_not_collide() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        let points = vec![
            Vec2::new(-16., -16.),
            Vec2::new(0., -16.),
            Vec2::new(16., -16.),
            Vec2::new(16., 0.),
            Vec2::new(16., 16.),
            Vec2::new(0., 16.),
            Vec2::new(-16., 16.),
            Vec2::new(-16., 0.),
        ];

        for point in points.iter() {
            assert_eq!(collider.intersect_point(*point).is_none(), true);
        }
    }

    #[test]
    fn test_intersect_point_does_collide() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        let points = vec![Vec2::new(4., 4.)];

        for point in points.iter() {
            assert_eq!(collider.intersect_point(*point).is_some(), true);
        }
    }

    #[test]
    fn test_intersect_segment_does_not_collide() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        assert_eq!(
            collider
                .intersect_segment(Vec2::new(-16., -16.), Vec2::new(32., 0.), 0., 0.)
                .is_none(),
            true
        );
        // no movement
        assert_eq!(
            collider
                .intersect_segment(Vec2::new(-16., -16.), Vec2::new(0., 0.), 0., 0.)
                .is_none(),
            true
        );
    }

    #[test]
    fn test_intersect_segment_does_collide() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        let point = Vec2::new(-16., 4.);
        let delta = Vec2::new(32., 0.);
        let result = collider.intersect_segment(point, delta, 0., 0.);
        assert_eq!(result.is_some(), true);

        let time = 0.25;
        let hit = result.unwrap();
        assert_eq!(hit.time, time);
        assert_eq!(hit.pos.x, point.x + delta.x * time);
        assert_eq!(hit.pos.y, point.y + delta.y * time);
        assert_eq!(hit.delta.x, (1.0 - time) * -delta.x);
        assert_eq!(hit.delta.y, (1.0 - time) * -delta.y);
        assert_eq!(hit.normal.x, -1.);
        assert_eq!(hit.normal.y, 0.);

        // static on top
        let result = collider.intersect_segment(Vec2::new(8., 8.), Vec2::new(0., 0.), 0., 0.);
        assert_eq!(result.is_some(), true);
        let hit = result.unwrap();
        assert_eq!(hit.delta.x, 0.);
        assert_eq!(hit.delta.y, 0.);

        // moving horizontal on top
        let result = collider.intersect_segment(Vec2::new(4., 8.), Vec2::new(18., 0.), 0., 0.);
        assert_eq!(result.is_some(), true);
        let hit = result.unwrap();
        assert_eq!(hit.delta.x, 0.);
        assert_eq!(hit.delta.y, 0.);
    }

    #[test]
    fn test_sweep_does_not_collide() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(64., -64.), Vec2::new(8., 8.));
        let delta = Vec2::new(0., 128.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_none(), true);
        assert_eq!(sweep.pos.x, collider2.center.x + delta.x);
        assert_eq!(sweep.pos.y, collider2.center.y + delta.y);
    }

    #[test]
    fn test_sweep_does_collide() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(0., -64.), Vec2::new(8., 8.));
        let delta = Vec2::new(0., 128.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_some(), true);
        // assert_eq!(sweep.pos.x, collider2.center.x + delta.x);
        // assert_eq!(sweep.pos.y, collider2.center.y + delta.y);
    }
}
