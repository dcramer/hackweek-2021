// based on http://noonat.github.io/intersect/

use bevy::{core::FixedTimestep, prelude::*};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::{
    components::{Collider, RigidBody},
    constants::GRAVITY,
};

use super::events::CollisionEvent;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<CollisionEvent>().add_system_set(
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

fn detect_collisions(
    mut ev_collision: EventWriter<CollisionEvent>,
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
            new_collider.update(body.position);

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

            ev_collision.send(CollisionEvent {
                entity,
                collider: *hit.collider,
            })
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

        collider.update(body.position);
    }
}
