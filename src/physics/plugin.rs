// based on http://noonat.github.io/intersect/

use bevy::math::Vec3Swizzles;
use bevy::{prelude::*, time::FixedTimestep};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::components::{Collider, RigidBody};

use super::events::CollisionEvent;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>().add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.))
                .with_system(detect_collisions.label("detect collisions"))
                .with_system(
                    apply_movements
                        .label("apply movements")
                        .after("detect collisions"),
                ),
        );
    }
}

fn detect_collisions(
    mut ev_collision: EventWriter<CollisionEvent>,
    collider_query: Query<(Entity, &Collider)>,
    mut rb_query: Query<(Entity, &mut RigidBody, &Collider)>,
    time: Res<Time>,
) {
    // first we need to compile a list of changes (compute all the movements and collisions)
    // then we will apply them
    for (entity, mut body, rb_collider) in rb_query.iter_mut() {
        let delta = (body.speed * time.delta_seconds()).xy().round();

        body.old_position = body.position;
        body.position.x += delta.x;
        body.position.y += delta.y;
        body.position = body.position.round();

        // dont compute collisions if we haven't moved
        if body.old_position == body.position {
            continue;
        }
        let was_on_ground = body.on_ground;
        let was_at_left_tile = body.at_left_tile;
        let was_at_right_tile = body.at_right_tile;
        let was_at_ceiling = body.at_ceiling;

        let mut new_collider = Collider::from_position(body.position, rb_collider.half);

        // update known knowns based on velocity
        // if body.speed.y < 0. || body.speed.y > 0. {
        //     println!("off ground1");
        //     body.on_ground = false;
        // }
        // if body.speed.y < 0. {
        //     body.at_ceiling = false;
        // }
        // if body.speed.x > 0. {
        //     body.at_left_tile = false;
        // }
        // if body.speed.x < 0. {
        //     body.at_right_tile = false;
        // }

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

            // it SEEMS like the delta is _off_ by roughly the size of _half_ but everythings computed from same relative positions.. so how?
            body.position.x += hit.delta.x;
            body.position.y += hit.delta.y;
            body.position = body.position.round();
            new_collider.update(body.position);

            body.at_left_tile = hit.delta.x > 0.;
            body.at_right_tile = hit.delta.x < 0.;
            if !body.at_right_tile && !body.at_left_tile {
                body.on_ground = hit.delta.y > 0.;
                body.at_ceiling = hit.delta.y < 0.;
            }

            ev_collision.send(CollisionEvent {
                entity,
                collider: *hit.collider,
            })
        }

        // now we need to test if we're still.. on the ground. is there a better way to do this??

        if was_at_ceiling && body.at_ceiling {
            let delta = Vec2::new(0., 2.0);
            let nearest = new_collider.sweep_into(
                collider_query
                    .iter()
                    .filter(|(e, _)| *e != entity)
                    .map(|(_, c)| c),
                delta,
            );
            body.at_ceiling = nearest.hit.is_some();
        }

        if was_on_ground && body.on_ground {
            let delta = Vec2::new(0., -1.0);
            let nearest = new_collider.sweep_into(
                collider_query
                    .iter()
                    .filter(|(e, _)| *e != entity)
                    .map(|(_, c)| c),
                delta,
            );
            body.on_ground = nearest.hit.is_some();
        }

        if was_at_right_tile && body.at_right_tile {
            let delta = Vec2::new(2., 0.0);
            let nearest = new_collider.sweep_into(
                collider_query
                    .iter()
                    .filter(|(e, _)| *e != entity)
                    .map(|(_, c)| c),
                delta,
            );
            body.at_right_tile = nearest.hit.is_some();
        }

        if was_at_left_tile && body.at_left_tile {
            let delta = Vec2::new(-2., 0.0);
            let nearest = new_collider.sweep_into(
                collider_query
                    .iter()
                    .filter(|(e, _)| *e != entity)
                    .map(|(_, c)| c),
                delta,
            );
            body.at_left_tile = nearest.hit.is_some();
        }

        if body.at_left_tile || body.at_right_tile {
            body.speed.x = 0.;
        }

        if body.on_ground && body.speed.y < 0. {
            body.speed.y = 0.;
        }

        if body.at_ceiling && body.speed.y > 0. {
            body.speed.y = 0.;
        }

        if was_on_ground != body.on_ground {
            println!(
                "ground: {:?} -> {:?} ({:?}",
                was_on_ground, body.on_ground, delta
            );
        }
    }
}

fn apply_movements(mut query: Query<(&mut Transform, &RigidBody, &mut Collider)>) {
    for (mut transform, body, mut collider) in query.iter_mut() {
        transform.translation = Vec3::new(body.position.x, body.position.y, body.position.z);
        transform.scale = Vec3::new(body.scale.x, body.scale.y, body.scale.z);

        collider.update(body.position);
    }
}
