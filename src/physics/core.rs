// based on http://noonat.github.io/intersect/

use bevy::prelude::*;

// use crate::constants::PLATFORM_THRESHOLD;
use crate::components::Collider;

const EPSILON: f32 = 1e-8;

pub struct Hit<'a> {
    pub collider: &'a Collider,
    /// the point of contact between the two objects (or an estimation of it, in some sweep tests).
    pub pos: Vec2,
    /// the overlap between the two objects, and is a vector that can be added to the colliding object’s position to move it back to a non-colliding state.
    pub delta: Vec2,
    /// the surface normal at the point of contact.
    pub normal: Vec2,
    /// defined for segment and sweep intersections, and is a fraction from 0 to 1 indicating how far along the line the collision occurred. (This is the tt value for the line equation L(t) = A + t(B - A)L(t)=A+t(B−A))
    pub time: f32,
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
    pub hit: Option<Hit<'a>>,
    /// the furthest point the object reached along the swept path before it hit something.
    pub pos: Vec2,
    /// a copy of hit.time, offset by epsilon, or 1.0 if the object didn’t hit anything during the sweep.
    pub time: f32,
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
        let dx = pos.x - self.pos.x;
        let px = self.half.x - dx.abs();
        if px <= 0. {
            return None;
        }

        let dy = pos.y - self.pos.y;
        let py = self.half.y - dy.abs();
        if py <= 0. {
            return None;
        }

        let mut hit = Hit::new(self);
        if px < py {
            let sx = dx.signum();
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.pos.x + (self.half.x * sx);
            hit.pos.y = pos.y;
        } else {
            let sy = dy.signum();
            hit.delta.y = py * sy;
            hit.normal.y = sy;
            hit.pos.x = pos.x;
            hit.pos.y = self.pos.y + (self.half.y * sy);
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
        let near_time_x = (self.pos.x - sign_x * (self.half.x + padding_x) - pos.x) * scale_x;
        let near_time_y = (self.pos.y - sign_y * (self.half.y + padding_y) - pos.y) * scale_y;
        let far_time_x = (self.pos.x + sign_x * (self.half.x + padding_x) - pos.x) * scale_x;
        let far_time_y = (self.pos.y + sign_y * (self.half.y + padding_y) - pos.y) * scale_y;

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
        let dx = other.pos.x - self.pos.x;
        let px = (other.half.x + self.half.x) - dx.abs();
        if px <= 0. {
            return None;
        }

        let dy = other.pos.y - self.pos.y;
        let py = (other.half.y + self.pos.y) - dy.abs();
        if py <= 0. {
            return None;
        }

        let mut hit = Hit::new(&self);
        if px < py {
            let sx = dx.signum();
            hit.delta.x = px * sx;
            hit.normal.x = sx;
            hit.pos.x = self.pos.x + (self.half.x * sx);
            hit.pos.y = other.pos.y;
        } else {
            let sy = dy.signum();
            hit.delta.y = py * sy;
            hit.normal.y = sy;
            hit.pos.x = other.pos.x;
            hit.pos.y = self.pos.y + (self.half.y * sy);
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
            sweep.pos.x = other.pos.x;
            sweep.pos.y = other.pos.y;
            if let Some(mut hit) = self.intersect(other) {
                hit.time = 0.;
                sweep.time = hit.time;
                sweep.hit = Some(hit);
            } else {
                sweep.time = 1.;
            }
            return sweep;
        }

        sweep.pos.x = other.pos.x + delta.x;
        sweep.pos.y = other.pos.y + delta.y;
        sweep.time = 1.;

        if let Some(mut hit) = self.intersect_segment(other.pos, delta, other.half.x, other.half.y)
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
            sweep.pos.x = other.pos.x + delta.x * sweep.time;
            sweep.pos.y = other.pos.y + delta.y * sweep.time;
            let direction = delta.clone();
            direction.normalize();
            hit.pos.x = (hit.pos.x + direction.x * other.half.x)
                .clamp(self.pos.x - self.half.x, self.pos.x + self.half.x);
            hit.pos.y = (hit.pos.y + direction.y * other.half.y)
                .clamp(self.pos.y - self.half.y, self.pos.y + self.half.y);
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
        nearest.pos = self.pos + delta;
        for collider in collider_iter {
            let sweep = collider.sweep(self, delta);
            if sweep.time < nearest.time {
                nearest = sweep;
            }
        }
        nearest
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
        assert_eq!(sweep.pos.x, collider2.pos.x + delta.x);
        assert_eq!(sweep.pos.y, collider2.pos.y + delta.y);
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
        let actor = Collider::new(Vec2::new(0., 33.), half);

        // flat line, no gaps, left to right (effectively starting at -16px)
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
        let hit = nearest.hit.unwrap();
        assert_eq!(hit.collider, &colliders[0]);
        assert_eq!(hit.delta.x, 0.);
        assert_eq!(hit.delta.y, 1.);
    }

    #[test]
    fn test_vertical_movement_no_gravity() {
        let half = Vec2::new(16., 16.);
        let actor = Collider::new(Vec2::new(0., 1063.), half);

        // straight vertical drop from large distance
        let colliders = vec![Collider::new(Vec2::ZERO, half)];
        let delta = Vec2::new(0., -1032.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        // assert_eq!(nearest.time, 1.);
        assert_eq!(nearest.hit.is_some(), true);
        let hit = nearest.hit.unwrap();
        assert_eq!(hit.collider, &colliders[0]);
        assert_eq!(hit.delta.x, 0.);
        assert_eq!(hit.delta.y, 1.);
    }

    #[test]
    fn test_jumping_movement_no_gravity() {
        let half = Vec2::new(16., 16.);
        let actor = Collider::new(Vec2::new(0., 33.), half);

        // an inverted L
        let colliders = vec![
            Collider::new(Vec2::ZERO, half),
            Collider::new(Vec2::new(32., 0.), half),
            Collider::new(Vec2::new(64., 0.), half),
            Collider::new(Vec2::new(64., 32.), half),
            Collider::new(Vec2::new(64., 64.), half),
        ];
        let delta = Vec2::new(72., 56.);
        let nearest = actor.sweep_into(colliders.iter(), delta);
        // assert_eq!(nearest.time, 1.);
        assert_eq!(nearest.hit.is_some(), true);
        let hit = nearest.hit.unwrap();
        assert_eq!(hit.collider, &colliders[3]);
        assert_eq!(hit.delta.x, -40.);
        assert_eq!(hit.delta.y, 0.);
    }
}
