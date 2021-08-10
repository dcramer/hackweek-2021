use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{Direction, Jumping, Player, PlayerReadyAttack, Projectile, Speed},
    constants::{JUMP_HEIGHT, JUMP_VELOCITY, SPRITE_SCALE, TIME_STEP},
    map::Map,
    Materials, WinSize,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_stage(
            "game_setup_actors",
            SystemStage::single(player_spawn.system()),
        )
        .add_system(player_movement.system())
        .add_system(player_jump.system())
        .add_system(projectile_movement.system())
        .add_system(player_attack.system());
    }
}

fn player_spawn(mut commands: Commands, materials: Res<Materials>, map: Res<Map>) {
    // spawn a sprite
    let spawn_pos = map.starting_positions[0];
    println!("Spawning player at {}, {}", spawn_pos.x, spawn_pos.y);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.player.clone(),
            // current sprite is 16x28
            transform: Transform::from_scale(Vec3::new(SPRITE_SCALE, SPRITE_SCALE * 0.57, 1.0)),
            ..Default::default()
        })
        .insert(Player::default())
        .insert(PlayerReadyAttack(true))
        .insert(Speed::default())
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic,
            // velocity: RigidBodyVelocity {
            //     linvel: Vec2::new(1.0, 2.0).into(),
            //     angvel: 0.2,
            // },
            forces: RigidBodyForces {
                gravity_scale: 10.,
                ..Default::default()
            },
            mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED).into(),
            position: Vec2::new(spawn_pos.x * SPRITE_SCALE, spawn_pos.y * SPRITE_SCALE).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(SPRITE_SCALE / 2., SPRITE_SCALE / 2.),
            material: ColliderMaterial {
                restitution: 0.,
                friction: 0.,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBodyPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(3));

    // spawn with default weapon
    // commands
    //     .spawn_bundle(SpriteBundle {
    //         material: materials.weapon_bow.clone(),
    //         transform: Transform {
    //             translation: Vec3::new(8.0, bottom + 75.0 / 4.0 + 10.0, EQUIPMENT_LAYER),
    //             scale: Vec3::new(1.5, 1.5, 1.0),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .insert(Weapon);
}

fn player_movement(
    kb: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(
        &Speed,
        &mut Player,
        &mut Transform,
        &mut RigidBodyPosition,
        With<Player>,
    )>,
) {
    if let Ok((speed, mut player, mut player_tf, mut player_pos, _)) = query.single_mut() {
        let move_x = if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
            -1.0
        } else if kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D) {
            1.0
        } else {
            0.
        };

        let new_x = player_pos.position.translation.x + move_x * speed.0 * TIME_STEP;
        if new_x < win_size.w / 2. && new_x > -win_size.w / 2. {
            player_pos.position.translation.x = new_x;
        }
        if move_x > 0.0 && player.facing != Direction::Right {
            player.facing = Direction::Right;
            player_tf.scale = Vec3::new(-player_tf.scale.x, player_tf.scale.y, player_tf.scale.z);
        } else if move_x < 0.0 && player.facing != Direction::Left {
            player.facing = Direction::Left;
            player_tf.scale = Vec3::new(-player_tf.scale.x, player_tf.scale.y, player_tf.scale.z);
        }
    }
}

fn player_jump(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &RigidBodyPosition, (With<Player>, Without<Jumping>))>,
    mut jumping_query: Query<(Entity, &mut RigidBodyPosition, &mut Jumping, With<Jumping>)>,
) {
    // can the player jump?
    if let Ok((player_entity, player_rbp, _)) = query.single_mut() {
        if (kb.pressed(KeyCode::Up) || kb.pressed(KeyCode::W) || kb.pressed(KeyCode::Space))
            && player_rbp.next_position.translation.y == player_rbp.position.translation.y
        {
            commands.entity(player_entity).insert(Jumping {
                height: JUMP_HEIGHT,
            });
        }
    }

    // currently jumping players
    for (jumping_entity, mut jumping_rbp, mut jumping, _) in jumping_query.iter_mut() {
        // TODO: how do we scale jump so its e.g. "3 sprites high"
        jumping.height -= JUMP_VELOCITY;
        let jump_y = if jumping.height < 0. {
            0.
        } else if jumping.height > JUMP_VELOCITY {
            JUMP_VELOCITY
        } else {
            JUMP_VELOCITY - jumping.height
        };
        let new_y = jumping_rbp.position.translation.y + jump_y;
        if new_y < win_size.h / 2. && new_y > -win_size.h / 2. {
            jumping_rbp.position.translation.y = new_y;
        }
        if jumping.height < 0.
            && jumping_rbp.next_position.translation.y == jumping_rbp.position.translation.y
        {
            commands.entity(jumping_entity).remove::<Jumping>();
        }
    }
}

fn player_attack(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    materials: Res<Materials>,
    mut query: Query<(
        &RigidBodyPosition,
        &mut PlayerReadyAttack,
        &Player,
        With<Player>,
    )>,
) {
    if let Ok((player_rbp, mut ready_attack, player, _)) = query.single_mut() {
        if ready_attack.0 && kb.pressed(KeyCode::Return) {
            let x = player_rbp.position.translation.x;
            let y = player_rbp.position.translation.y;
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.projectile.clone(),
                    transform: Transform::from_scale(Vec3::new(
                        if player.facing == Direction::Right {
                            -1.
                        } else {
                            1.
                        } * SPRITE_SCALE
                            / 2.0,
                        SPRITE_SCALE / 2.0,
                        1.,
                    )),
                    ..Default::default()
                })
                .insert(Projectile {
                    direction: player.facing,
                })
                .insert(Speed(50.))
                .insert_bundle(ColliderBundle {
                    shape: ColliderShape::ball(SPRITE_SCALE / 4.),
                    position: Vec2::new(x, y).into(),
                    material: ColliderMaterial {
                        restitution: 0.,
                        friction: 0.,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ColliderPositionSync::Discrete)
                .insert(ColliderDebugRender::with_id(2));
            ready_attack.0 = false;
        }

        if kb.just_released(KeyCode::Return) {
            ready_attack.0 = true;
        }
    }
}

fn projectile_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(
        Entity,
        &Projectile,
        &Speed,
        &mut ColliderPosition,
        With<Projectile>,
    )>,
) {
    for (proj_entity, projectile, proj_speed, mut proj_cp, _) in query.iter_mut() {
        let translation = &mut proj_cp.0.translation;
        if projectile.direction == Direction::Right {
            translation.x += proj_speed.0 * TIME_STEP;
        } else if projectile.direction == Direction::Left {
            translation.x -= proj_speed.0 * TIME_STEP;
        } else if projectile.direction == Direction::Up {
            translation.y += proj_speed.0 * TIME_STEP;
        } else if projectile.direction == Direction::Down {
            translation.y -= proj_speed.0 * TIME_STEP;
        }
        if translation.x > win_size.w / 2.
            || translation.x < -win_size.w / 2.
            || translation.y > win_size.h / 2.
            || translation.y < -win_size.h / 2.
        {
            commands.entity(proj_entity).despawn();
        }
    }
}
