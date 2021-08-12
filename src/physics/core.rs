// based on http://noonat.github.io/intersect/

use bevy::{core::FixedTimestep, prelude::*};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Collider, RigidBody},
    constants::GRAVITY,
};

const EPSILON: f32 = 1e-8;

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
            let sx = dx.signum();
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.center.x + (self.half.x * sx);
            hit.pos.y = pos.y;
        } else {
            let sy = dy.signum();
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
        let sign_x = scale_x.signum();
        let sign_y = scale_y.signum();
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
        hit.delta.y = (1. - hit.time) * -delta.y;
        hit.pos.x = pos.x + delta.x * hit.time;
        hit.pos.y = pos.y + delta.y * hit.time;
        Some(hit)
    }

    /// detect an intersection with a static collider
    fn intersect<'a, 'b>(&'a self, other: &'b Collider) -> Option<Hit<'a>> {
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
            let sx = dx.signum();
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.center.x + (self.half.x * sx);
            hit.pos.y = other.center.y;
        } else {
            let sy = dy.signum();
            hit.delta.y = py * sy;
            hit.normal.y = sy;
            hit.pos.x = other.center.x;
            hit.pos.y = self.center.y + (self.half.y * sy);
        }
        Some(hit)
    }

    /// detect an intersection with a dynamic collider
    // TODO: Hit having a reference to self is not very useful. Ideally it'd be set to `other` here, as we're
    // sweeping from `self` into `other`, so knowing _what_ we hit is more valuable.
    // we have manually set it for now but API needs some work
    pub fn sweep<'a, 'b>(&'a self, other: &'b Collider, delta: Vec2) -> Sweep<'a> {
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

        sweep.pos.x = other.center.x + delta.x;
        sweep.pos.y = other.center.y + delta.y;
        sweep.time = 1.;

        if let Some(mut hit) =
            self.intersect_segment(other.center, delta, other.half.x, other.half.y)
        {
            // TODO: all this behavior is weird idk what im doing.. really only using the top-only collider
            if delta.x < 0. && !self.left {
                return sweep;
            } else if delta.x > 0. && !self.right {
                return sweep;
            } else if delta.y > 0. && !self.bottom {
                return sweep;
            } else if delta.y < 0. && !self.top {
                return sweep;
            }
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
        }
        sweep
    }

    pub fn sweep_into<'a, 'b>(
        &'a self,
        collider_iter: impl Iterator<Item = &'b Collider>,
        delta: Vec2,
    ) -> Sweep<'b> {
        let mut nearest = Sweep::default();
        nearest.pos = self.center + delta;
        for collider in collider_iter {
            let sweep = collider.sweep(self, delta);
            if sweep.time < nearest.time {
                nearest = sweep;
            }
        }
        nearest
    }
}

fn detect_collisions(
    collider_query: Query<(Entity, &Collider)>,
    mut rb_query: Query<(Entity, &mut RigidBody, &Collider)>,
    time: Res<Time>,
) {
    // first we need to compile a list of changes (compute all the movements and collisions)
    // then we will apply them
    for (entity, mut body, rb_collider) in rb_query.iter_mut() {
        let was_on_ground = body.on_ground;

        let delta = Vec2::new(
            body.speed.x * time.delta_seconds(),
            body.speed.y * time.delta_seconds(),
        )
        .round();

        body.old_position = body.position;
        body.position.x += delta.x;
        body.position.y += delta.y;
        body.position = body.position.round();

        // dont compute collisions if we haven't moved
        if body.old_position == body.position {
            continue;
        }

        let mut new_collider = Collider::from_position(body.position, rb_collider.half);

        // update known knowns based on velocity
        // if body.speed.y < 0. || body.speed.y > 0. {
        //     println!("off ground1");
        //     body.on_ground = false;
        // }
        if body.speed.y < 0. {
            body.at_ceiling = false;
        }
        if body.speed.x > 0. {
            body.at_left_tile = false;
        }
        if body.speed.x < 0. {
            body.at_right_tile = false;
        }

        let nearest = new_collider.sweep_into(
            collider_query
                .iter()
                .filter(|(e, _)| *e != entity)
                .map(|(_, c)| c),
            delta,
        );
        if let Some(hit) = nearest.hit {
            // TODO: this math isnt precise
            // - sometimes it pushes you up from the ground (vs on top of the collider)
            // - it still has floating errors, so its never even "on top" of the collider
            // - we hit left/right tiles (toe stubbing) with gravity and it causes movements that are not desired

            body.position.x += hit.delta.x;
            body.position.y += hit.delta.y;
            body.position = body.position.round();
            new_collider.center_from(body.position);

            if (hit.delta.x < 0. && body.speed.x > 0.) || (hit.delta.x > 0. && body.speed.x < 0.) {
                body.speed.x = 0.;
                // body.at_left_tile = hit.delta.x > 0.;
                // body.at_right_tile = hit.delta.x < 0.;
            }

            if hit.delta.y > 0. {
                body.speed.y = 0.;
                body.on_ground = true;
            }

            if hit.delta.y < 0. && body.speed.y > 0. {
                body.speed.y = GRAVITY * time.delta_seconds();
                body.on_ground = false;
                body.at_ceiling = true;
            }
        }

        // now we need to test if we're still.. on the ground. is there a better way to do this??
        if was_on_ground && body.on_ground {
            let delta = Vec2::new(0., -2.0);
            let nearest = new_collider.sweep_into(
                collider_query
                    .iter()
                    .filter(|(e, _)| *e != entity)
                    .map(|(_, c)| c),
                delta,
            );
            body.on_ground = nearest.hit.is_some();
        }
    }
}

fn apply_movements(mut query: Query<(&mut Transform, &RigidBody, &mut Collider)>) {
    for (mut transform, body, mut collider) in query.iter_mut() {
        transform.translation = Vec3::new(body.position.x, body.position.y, body.position.z);
        transform.scale = Vec3::new(body.scale.x, body.scale.y, body.scale.z);

        collider.center_from(body.position);
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
        assert_eq!(hit.collider, &collider);
        assert_eq!(hit.time, time);
        assert_eq!(hit.pos.x, point.x + delta.x * time);
        assert_eq!(hit.pos.y, point.y + delta.y * time);
        assert_eq!(hit.delta.x, (1.0 - time) * -delta.x);
        assert_eq!(hit.delta.y, (1.0 - time) * -delta.y);
        assert_eq!(hit.normal.x, -1.);
        assert_eq!(hit.normal.y, 0.);
    }

    #[test]
    fn test_intersect_segment_sets_hit_time_to_zero_when_segment_starts_inside_box() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        let point = Vec2::new(-4., 4.);
        let delta = Vec2::new(32., 0.);
        let result = collider.intersect_segment(point, delta, 0., 0.);
        assert_eq!(result.is_some(), true);

        let hit = result.unwrap();
        assert_eq!(hit.collider, &collider);
        assert_eq!(hit.time, 0.);
        assert_eq!(hit.pos.x, -4.);
        assert_eq!(hit.pos.y, 4.);
        assert_eq!(hit.delta.x, -delta.x);
        assert_eq!(hit.delta.y, -delta.y);
        assert_eq!(hit.normal.x, -1.);
        assert_eq!(hit.normal.y, 0.);
    }

    #[test]
    fn test_intersect_segment_should_add_padding_to_half_size_box() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(8., 8.));
        let point = Vec2::new(-16., 4.);
        let delta = Vec2::new(32., 0.);
        let padding = 4.;
        let result = collider.intersect_segment(point, delta, padding, padding);
        assert_eq!(result.is_some(), true);

        let time = 0.125;
        let hit = result.unwrap();
        assert_eq!(hit.collider, &collider);
        assert_eq!(hit.time, time);
        assert_eq!(hit.pos.x, point.x + delta.x * time);
        assert_eq!(hit.pos.y, point.y + delta.y * time);
        assert_eq!(hit.delta.x, (1.0 - time) * -delta.x);
        assert_eq!(hit.delta.y, (1.0 - time) * -delta.y);
        assert_eq!(hit.normal.x, -1.);
        assert_eq!(hit.normal.y, 0.);
    }

    #[test]
    fn test_intersect_segment_should_have_consistent_results_in_both_directions() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(32., 32.));
        let far_pos = Vec2::new(-64., 0.);
        let far_to_near_delta = Vec2::new(-32., 0.);
        assert_eq!(
            collider
                .intersect_segment(far_pos, far_to_near_delta, 0., 0.)
                .is_none(),
            true
        );

        let near_pos = Vec2::new(32., 0.);
        let near_to_far_delta = Vec2::new(32., 0.);
        assert_eq!(
            collider
                .intersect_segment(near_pos, near_to_far_delta, 0., 0.)
                .is_none(),
            true
        );
    }

    #[test]
    fn test_intersect_segment_should_work_when_axis_aligned() {
        let collider = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let pos = Vec2::new(-32., 0.);
        let delta = Vec2::new(64., 0.);
        let result = collider.intersect_segment(pos, delta, 0., 0.);
        assert_eq!(result.is_some(), true);

        let hit = result.unwrap();
        assert_eq!(hit.time, 0.25);
        assert_eq!(hit.normal.x, -1.);
        assert_eq!(hit.normal.y, 0.);
    }

    #[test]
    fn test_sweep_does_not_collide() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(64., -64.), Vec2::new(8., 8.));
        let delta = Vec2::new(0., 128.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_none(), true);
        assert_eq!(sweep.time, 1.);
        assert_eq!(sweep.pos.x, collider2.center.x + delta.x);
        assert_eq!(sweep.pos.y, collider2.center.y + delta.y);
    }

    #[test]
    fn test_sweep_does_collide_vertical() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(0., -64.), Vec2::new(8., 8.));
        let delta = Vec2::new(0., 128.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_some(), true);

        let hit = sweep.hit.unwrap();
        assert_eq!(hit.collider, &collider1);
        assert_eq!(hit.delta.x, -0.);
        assert_eq!(hit.delta.y, -88.);
    }

    #[test]
    fn test_sweep_does_collide_horizontal() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(-64., 0.), Vec2::new(8., 8.));
        let delta = Vec2::new(128., 0.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_some(), true);

        let hit = sweep.hit.unwrap();
        assert_eq!(hit.collider, &collider1);
        assert_eq!(hit.delta.x, -88.);
        assert_eq!(hit.delta.y, -0.);
    }

    #[test]
    fn test_sweep_does_collide_diagonal() {
        let collider1 = Collider::new(Vec2::ZERO, Vec2::new(16., 16.));
        let collider2 = Collider::new(Vec2::new(-64., -64.), Vec2::new(8., 8.));
        let delta = Vec2::new(128., 128.);
        let sweep = collider1.sweep(&collider2, delta);
        assert_eq!(sweep.hit.is_some(), true);

        let hit = sweep.hit.unwrap();
        assert_eq!(hit.collider, &collider1);
        assert_eq!(hit.delta.x, -88.);
        assert_eq!(hit.delta.y, -88.);
    }

    #[test]
    fn test_sweep_into_does_collide() {
        let actor = Collider::new(Vec2::new(64., -64.), Vec2::new(8., 8.));

        let colliders = vec![
            Collider::new(Vec2::ZERO, Vec2::new(16., 16.)),
            Collider::new(Vec2::new(0., -64.), Vec2::new(8., 8.)),
        ];
        let delta = Vec2::new(-64., 128.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        assert_eq!(nearest.time, 0.625);
        assert_eq!(nearest.hit.is_some(), true);

        let hit = nearest.hit.unwrap();
        assert_eq!(*hit.collider, colliders[0]);
    }

    // XXX: I can't seem to debug math at all (bad brain) so lets write tests to check character movement
    #[test]
    fn test_horizontal_movement_no_gravity() {
        let half = Vec2::new(16., 16.);
        // start at the "left most" tile, with a vertical offset to prevent ground collision
        let actor = Collider::new(Vec2::new(16., 17.), half);

        let colliders = vec![
            Collider::new(Vec2::ZERO, half),
            Collider::new(Vec2::new(16., 0.), half),
            Collider::new(Vec2::new(48., 0.), half),
        ];
        let delta = Vec2::new(4., 0.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        assert_eq!(nearest.time, 1.);
        assert_eq!(nearest.hit.is_some(), false);

        let delta = Vec2::new(-4., 0.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        assert_eq!(nearest.time, 1.);
        assert_eq!(nearest.hit.is_some(), false);

        let delta = Vec2::new(0., -2.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        // assert_eq!(nearest.time, 1.);
        assert_eq!(nearest.hit.is_some(), true);
    }
}
