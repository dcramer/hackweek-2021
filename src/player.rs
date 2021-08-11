use bevy::prelude::*;

use crate::{
    components::{Direction, Player, PlayerReadyAttack, PlayerState, Projectile, RigidBody, Speed},
    constants::{GRAVITY, MAX_FALLING_SPEED, PLATFORM_THRESHOLD, SPRITE_SCALE},
    map::Map,
    resources::{CharacterAnimation, Materials, WinSize},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_stage(
            "game_setup_actors",
            SystemStage::single(player_spawn.system()),
        )
        .add_system(player_movement.system())
        .add_system(projectile_movement.system())
        .add_system(player_attack.system());
    }
}

fn player_spawn(mut commands: Commands, char_anim: Res<CharacterAnimation>, map: Res<Map>) {
    let spawn_pos = map.starting_positions[0];
    let transform = Transform {
        translation: Vec3::new(spawn_pos.x, spawn_pos.y, 10.),
        scale: Vec3::new(1.5, 1.5, 1.),
        ..Default::default()
    };

    println!("Spawning player at {}, {}", spawn_pos.x, spawn_pos.y);
    commands
        .spawn_bundle(SpriteBundle {
            material: char_anim.idle_f0.clone(),
            // current sprite is 16x28
            transform,
            ..Default::default()
        })
        .insert(Player::default())
        .insert(PlayerReadyAttack(true))
        .insert(RigidBody::from_transform(transform, 24., 30.))
        .insert(Speed::new(240.0, 0.0));

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

fn player_animation(
    char_anim: Res<CharacterAnimation>,
    mut query: Query<(&mut Timer, &Player, &mut Sprite, With<Player>)>,
) {
    //     for (mut timer, player, mut material, _) in query.iter_mut() {
    //         timer.tick(time.delta());
    //         if timer.finished() {
    //             if player.state == PlayerState::Stand {
    //                 material.texture = char_anim.idle_f0.clone();
    //             } else {
    //                 material.texture = char_anim.run_f0.clone();
    //             }
    //         }
    //     }
    // }
}

fn player_movement(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Speed, &mut Player, &mut RigidBody, With<Player>)>,
) {
    if let Ok((speed, mut player, mut rigidbody, _)) = query.single_mut() {
        match player.state {
            PlayerState::Stand => {
                rigidbody.speed = Vec3::ZERO;

                if !rigidbody.on_ground {
                    player.state = PlayerState::Jump;
                    return;
                }

                // if left or right pressed, not both
                if (kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A))
                    != (kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D))
                {
                    player.state = PlayerState::Walk;
                    return;
                // if jump pressed
                } else if kb.pressed(KeyCode::Space) {
                    rigidbody.speed.y = player.jump_speed;
                    player.state = PlayerState::Jump;
                } else if kb.pressed(KeyCode::Down) || kb.pressed(KeyCode::S) {
                    if rigidbody.on_platform {
                        rigidbody.position.y -= PLATFORM_THRESHOLD;
                    }
                }
            }
            PlayerState::Walk => {
                // if both left and right pressed, or no keys pressed, stop
                if (kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A))
                    == (kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D))
                {
                    player.state = PlayerState::Stand;
                    rigidbody.speed = Vec3::ZERO;
                // go right
                } else if kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D) {
                    if rigidbody.pushes_right_tile {
                        rigidbody.speed.x = 0.;
                    } else {
                        rigidbody.speed.x = speed.0.x;
                    }
                    rigidbody.scale.x = rigidbody.scale.x.abs();
                    player.facing = Direction::Right;
                // go left
                } else if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
                    if rigidbody.pushes_left_tile {
                        rigidbody.speed.x = 0.;
                    } else {
                        rigidbody.speed.x = -speed.0.x;
                    }
                    rigidbody.scale.x = -rigidbody.scale.x.abs();
                    player.facing = Direction::Left;
                } else if kb.pressed(KeyCode::Down) || kb.pressed(KeyCode::S) {
                    if rigidbody.on_platform {
                        rigidbody.position.y -= PLATFORM_THRESHOLD;
                    }
                }
                // if theres no tile to walk on, fall
                if kb.pressed(KeyCode::Space) {
                    rigidbody.speed.y = player.jump_speed;
                    player.state = PlayerState::Jump;
                } else if !rigidbody.on_ground {
                    player.state = PlayerState::Jump;
                }
            }
            PlayerState::Jump => {
                rigidbody.speed.y += GRAVITY * time.delta_seconds();
                if rigidbody.speed.y < MAX_FALLING_SPEED {
                    rigidbody.speed.y = MAX_FALLING_SPEED;
                }

                if !kb.pressed(KeyCode::Space) && rigidbody.speed.y > 0. {
                    if rigidbody.speed.y > player.min_jump_speed {
                        rigidbody.speed.y = player.min_jump_speed;
                    }
                }

                // stop moving
                if (kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A))
                    == (kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D))
                {
                    rigidbody.speed.x = 0.;
                // go right
                } else if kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D) {
                    if rigidbody.pushes_right_tile {
                        rigidbody.speed.x = 0.;
                    } else {
                        rigidbody.speed.x = speed.0.x;
                    }
                    rigidbody.scale.x = rigidbody.scale.x.abs();
                    player.facing = Direction::Right;
                // go left
                } else if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
                    if rigidbody.pushes_left_tile {
                        rigidbody.speed.x = 0.;
                    } else {
                        rigidbody.speed.x = -speed.0.x;
                    }
                    rigidbody.scale.x = -rigidbody.scale.x.abs();
                    player.facing = Direction::Left;
                }

                if rigidbody.on_ground {
                    if (kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A))
                        == (kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D))
                    {
                        player.state = PlayerState::Stand;
                        rigidbody.speed = Vec3::ZERO;
                    } else {
                        player.state = PlayerState::Walk;
                        rigidbody.speed.y = 0.;
                    }
                }
            }
        }
    }
}

fn player_attack(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    materials: Res<Materials>,
    mut query: Query<(&Transform, &mut PlayerReadyAttack, &Player, With<Player>)>,
) {
    if let Ok((player_tf, mut ready_attack, player, _)) = query.single_mut() {
        if ready_attack.0 && kb.pressed(KeyCode::Return) {
            let x = player_tf.translation.x;
            let y = player_tf.translation.y;
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.projectile.clone(),
                    transform: Transform {
                        translation: Vec3::new(x, y, 1.),
                        scale: Vec3::new(
                            if player.facing == Direction::Right {
                                -1.
                            } else {
                                1.
                            } * SPRITE_SCALE
                                / 2.0,
                            SPRITE_SCALE / 2.0,
                            1.,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Projectile {
                    direction: player.facing,
                })
                .insert(Speed::new(1000., 1000.));
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
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Projectile,
        &Speed,
        &mut Transform,
        With<Projectile>,
    )>,
) {
    for (proj_entity, projectile, proj_speed, mut proj_tf, _) in query.iter_mut() {
        let translation = &mut proj_tf.translation;
        if projectile.direction == Direction::Right {
            translation.x += proj_speed.0.x * time.delta_seconds();
        } else if projectile.direction == Direction::Left {
            translation.x -= proj_speed.0.x * time.delta_seconds();
        } else if projectile.direction == Direction::Up {
            translation.y += proj_speed.0.y * time.delta_seconds();
        } else if projectile.direction == Direction::Down {
            translation.y -= proj_speed.0.y * time.delta_seconds();
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
