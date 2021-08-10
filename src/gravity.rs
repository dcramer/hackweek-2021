use std::cmp;

use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;

use crate::{
    components::{Collider, Gravity, Velocity},
    constants::TIME_STEP,
    resources::WinSize,
};

const TERMINAL_VELOCITY: f32 = -1000.0;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(gravity.system());
    }
}

fn gravity(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut set: QuerySet<(
        Query<(Entity, &mut Transform, &Collider, &mut Velocity, &Gravity)>,
        Query<(&Transform, &Collider)>,
    )>,
) {
    let bottom = -win_size.h / 2.;

    for (entity, mut tf, collider, mut velocity, gravity) in set.q0_mut().iter_mut() {
        let new_velocity = if velocity.0 - gravity.0 < TERMINAL_VELOCITY {
            TERMINAL_VELOCITY
        } else {
            velocity.0 - gravity.0
        };
        let new_y = tf.translation.y + new_velocity * TIME_STEP;
        let new_translation = Vec3::new(tf.translation.x, new_y, tf.translation.z);
        let will_collide = set
            .q1()
            .iter()
            .find(|(co_tf, co_co)| {
                collide(
                    new_translation,
                    collider.size,
                    co_tf.translation,
                    co_co.size,
                )
                .is_some()
            })
            .is_some();
        if will_collide {
            commands.entity(entity).remove::<Velocity>();
        } else {
            let new_y = tf.translation.y + velocity.0 * TIME_STEP;
            if new_y < win_size.h / 2. && new_y > bottom {
                tf.translation.y = new_y;
            }
            velocity.0 = new_velocity;
        }
    }
}
