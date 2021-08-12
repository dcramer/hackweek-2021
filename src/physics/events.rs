use bevy::prelude::*;

use crate::components::Collider;

pub struct CollisionEvent {
    pub entity: Entity,
    pub collider: Collider,
}
