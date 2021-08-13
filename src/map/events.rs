use bevy::prelude::*;

use crate::physics::CollisionEvent;

pub fn tile_collision_listener(mut events: EventReader<CollisionEvent>) {
    for ev in events.iter() {
        info!("collision {:?} with {:?}", ev.entity, ev.collider);
    }
}
