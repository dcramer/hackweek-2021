// Based on: https://gamedevelopment.tutsplus.com/tutorials/basic-2d-platformer-physics-part-2--cms-25922

use bevy::{core::FixedTimestep, prelude::*};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::components::Collider;

pub struct ColliderDebugger;

pub struct ColliderMaterials {
    collider_border: Handle<ColorMaterial>,
}

pub struct DebugPhysicsPlugin;

impl Plugin for DebugPhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_debugger.system())
            .add_system(debug_colliders.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0 / 60.))
                    .with_system(
                        debug_colliders
                            .system()
                            .label("debug colliders")
                            .after("apply movements"),
                    ),
            );
    }
}

fn setup_debugger(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(ColliderMaterials {
        collider_border: materials.add(Color::rgba(1., 0., 0., 0.5).into()),
    });
}

fn debug_colliders(
    mut commands: Commands,
    materials: Res<ColliderMaterials>,
    mut query: Query<(Entity, &Collider, Without<ColliderDebugger>)>,
) {
    for (entity, collider, _) in query.iter_mut() {
        let half_y = collider.half.y / 2.;
        let half_x = collider.half.x / 2.;
        commands
            .entity(entity)
            .with_children(|parent| {
                // top
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        // left
                        0., // top
                        half_y, 500.,
                    )),
                    sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
                    ..Default::default()
                });
                // bottom
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        // left
                        0., // bottom
                        -half_y, 500.,
                    )),
                    sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
                    ..Default::default()
                });
                // left
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        // left
                        -half_x, // bottom
                        0., 500.,
                    )),
                    sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
                    ..Default::default()
                });
                // right
                parent.spawn_bundle(SpriteBundle {
                    material: materials.collider_border.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        // right
                        half_x, // bottom
                        0., 500.,
                    )),
                    sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
                    ..Default::default()
                });
            })
            .insert(ColliderDebugger);
    }
}
