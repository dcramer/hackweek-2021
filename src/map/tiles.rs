use bevy::prelude::*;

use crate::physics::CollisionEvent;

pub fn tile_collision_listener(mut events: EventReader<CollisionEvent>) {
    for my_event in events.iter() {
        info!("collision {:?}", my_event.entity);
    }
}
