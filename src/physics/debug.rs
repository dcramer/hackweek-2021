// Based on: https://gamedevelopment.tutsplus.com/tutorials/basic-2d-platformer-physics-part-2--cms-25922

use bevy::{prelude::*, time::FixedTimestep};

// use crate::constants::PLATFORM_THRESHOLD;
use crate::components::Collider;

#[derive(Component)]
pub struct HasDebugCollider;

#[derive(Resource)]
pub struct ColliderMaterials {
    collider_border: Handle<ColorMaterial>,
}

pub struct DebugPhysicsPlugin;

impl Plugin for DebugPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_debugger)
            //.add_system(insert_debug_colliders)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0 / 30.))
                    .with_system(
                        debug_colliders
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

// TODO: this has two problems
// 1. its not absolutely positioning the colliders where they should be (due to parent/child)
// 2. the scale is wrong due to parent/child (its inferring scale)
fn debug_colliders(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Collider, Without<HasDebugCollider>)>,
) {
    for (entity, transform, collider, _) in query.iter_mut() {
        let rel_scale = Vec3::new(1. / transform.scale.x, 1. / transform.scale.y, 1.);
        let rel_scale_v2 = Vec2::new(rel_scale.x, rel_scale.y);
        let rel_half = collider.half * rel_scale_v2;
        // compute the relative offset from the bottom left (transform.position) to collider.center
        // this allows us to find the bottom left corner of the collider, relative to the entity

        // 0 1 2 3 4
        //           4
        //   - - -   3
        //   - C -   2
        //   - - -   1
        //           0

        // compute the delta between bottom left of collider
        commands
            .entity(entity)
            .insert(HasDebugCollider)
            .with_children(|parent| {
                // top
                parent.spawn(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            0.,         // center
                            rel_half.y, // top
                            500.,
                        ),
                        scale: rel_scale,
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(collider.half.x * 2., 1.)),
                        color: Color::rgba(1., 0., 0., 0.5),
                        ..default()
                    },
                    ..default()
                });
                // bottom
                parent.spawn(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            0.,          // center
                            -rel_half.y, // bottom
                            500.,
                        ),
                        scale: rel_scale,
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(collider.half.x * 2., 1.)),
                        color: Color::rgba(1., 0., 0., 0.5),
                        ..default()
                    },
                    ..default()
                });
                // left
                parent.spawn(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            -rel_half.x, // left
                            0.,          // center
                            500.,
                        ),
                        scale: rel_scale,
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1., collider.half.y * 2.)),
                        color: Color::rgba(1., 0., 0., 0.5),
                        ..default()
                    },
                    ..default()
                });
                // right
                parent.spawn(SpriteBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            rel_half.x, // right
                            0.,         // center
                            500.,
                        ),
                        scale: rel_scale,
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1., collider.half.y * 2.)),
                        color: Color::rgba(1., 0., 0., 0.5),
                        ..default()
                    },
                    ..default()
                });
            });
    }
}

// fn update_debug_colliders(
//     mut commands: Commands,
//     materials: Res<ColliderMaterials>,
//     mut query: Query<(Entity, &DebugCollider)>,
// ) {
//     for (entity, debug_collider) in query.iter_mut() {
//         commands.entity(debug_collider.entity);
//         let half_y = collider.half.y / 2.;
//         let half_x = collider.half.x / 2.;
//         commands.entity(entity).with_children(|parent| {
//             // top
//             parent.spawn(SpriteBundle {
//                 material: materials.collider_border.clone(),
//                 transform: Transform::from_translation(Vec3::new(
//                     // left
//                     0., // top
//                     half_y, 500.,
//                 )),
//                 sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
//                 ..default()
//             });
//             // bottom
//             parent.spawn(SpriteBundle {
//                 material: materials.collider_border.clone(),
//                 transform: Transform::from_translation(Vec3::new(
//                     // left
//                     0., // bottom
//                     -half_y, 500.,
//                 )),
//                 sprite: Sprite::new(Vec2::new(collider.half.x, 1.0)),
//                 ..default()
//             });
//             // left
//             parent.spawn(SpriteBundle {
//                 material: materials.collider_border.clone(),
//                 transform: Transform::from_translation(Vec3::new(
//                     // left
//                     -half_x, // bottom
//                     0., 500.,
//                 )),
//                 sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
//                 ..default()
//             });
//             // right
//             parent.spawn(SpriteBundle {
//                 material: materials.collider_border.clone(),
//                 transform: Transform::from_translation(Vec3::new(
//                     // right
//                     half_x, // bottom
//                     0., 500.,
//                 )),
//                 sprite: Sprite::new(Vec2::new(1.0, collider.half.y)),
//                 ..default()
//             });
//         });
//     }
// }
