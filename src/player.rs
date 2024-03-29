use bevy::prelude::*;

use crate::{
    components::{
        Collider, Direction, Player, PlayerBundle, PlayerReadyAttack, PlayerState, Projectile,
        RigidBody, Speed,
    },
    constants::{GRAVITY, MAX_FALLING_SPEED, PLATFORM_THRESHOLD, SPRITE_SCALE},
    map::Map,
    resources::WinSize,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_stage("game_setup_actors", SystemStage::single(player_spawn))
            .add_system(player_movement)
            .add_system(projectile_movement)
            .add_system(player_attack);
    }
}

fn player_spawn(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let spawn_pos = map.starting_positions[0];
    let transform = Transform {
        translation: Vec3::new(spawn_pos.x, spawn_pos.y, 10.),
        scale: Vec3::new(1., 1., 1.),
        ..default()
    };

    println!("Spawning player at {}, {}", spawn_pos.x, spawn_pos.y);
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("anim/idle/knight_m_idle_anim_f0.png"),
            // sprite is 16x20
            // scaled to 24x30
            transform,
            sprite: Sprite {
                custom_size: Some(Vec2::new(16., 20.)),
                ..default()
            },
            ..default()
        })
        .insert(PlayerBundle::default())
        .insert(RigidBody::from_transform(transform))
        .insert(Collider::from_position(
            transform.translation,
            Vec2::new(8., 10.),
        ));

    // spawn with default weapon
    // commands
    //     .spawn(SpriteBundle {
    //         material: materials.weapon_bow.clone(),
    //         transform: Transform {
    //             translation: Vec3::new(8.0, bottom + 75.0 / 4.0 + 10.0, EQUIPMENT_LAYER),
    //             scale: Vec3::new(1.5, 1.5, 1.0),
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .insert(Weapon);
}

fn player_movement(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Speed, &mut Player, &mut RigidBody, With<Player>)>,
) {
    let (speed, mut player, mut rigidbody, _) = query.single_mut();
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
            } else if kb.pressed(KeyCode::Space) && !rigidbody.at_ceiling {
                rigidbody.speed.y = player.jump_speed;
                player.state = PlayerState::Jump;
            // if drop pressed
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
                if rigidbody.at_right_tile {
                    rigidbody.speed.x = 0.;
                } else {
                    rigidbody.speed.x = speed.0.x;
                }
                rigidbody.scale.x = rigidbody.scale.x.abs();
                player.facing = Direction::Right;
            // go left
            } else if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
                if rigidbody.at_left_tile {
                    rigidbody.speed.x = 0.;
                } else {
                    rigidbody.speed.x = -speed.0.x;
                }
                rigidbody.scale.x = -rigidbody.scale.x.abs();
                player.facing = Direction::Left;
            // if drop pressed
            } else if kb.pressed(KeyCode::Down) || kb.pressed(KeyCode::S) {
                if rigidbody.on_platform {
                    rigidbody.position.y -= PLATFORM_THRESHOLD;
                }
            }
            // if theres no tile to walk on, fall
            if kb.pressed(KeyCode::Space) && !rigidbody.at_ceiling {
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

            if rigidbody.at_ceiling || (!kb.pressed(KeyCode::Space) && rigidbody.speed.y > 0.) {
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
                if rigidbody.at_right_tile {
                    rigidbody.speed.x = 0.;
                } else {
                    rigidbody.speed.x = speed.0.x;
                }
                rigidbody.scale.x = rigidbody.scale.x.abs();
                player.facing = Direction::Right;
            // go left
            } else if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
                if rigidbody.at_left_tile {
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

fn player_attack(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    mut query: Query<(&Transform, &mut PlayerReadyAttack, &Player, With<Player>)>,
    asset_server: Res<AssetServer>,
) {
    let (player_tf, mut ready_attack, player, _) = query.single_mut();
    if ready_attack.0 && kb.pressed(KeyCode::Return) {
        let x = player_tf.translation.x;
        let y = player_tf.translation.y;
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("bullet.png"),
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
                    ..default()
                },
                ..default()
            })
            .insert(Projectile {
                direction: player.facing,
            })
            .insert(Speed::new(1000., 1000.));
        ready_attack.0 = false;

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
