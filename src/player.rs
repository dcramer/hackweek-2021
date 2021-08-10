use bevy::prelude::*;

use crate::{
    components::{Direction, Gravity, Player, PlayerReadyAttack, Projectile, Speed, Velocity},
    constants::{SPRITE_SCALE, TIME_STEP},
    display::normalize_pos,
    map::Map,
    resources::{Materials, WinSize},
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
    let spawn_pos = normalize_pos(map.starting_positions[0]);
    println!("Spawning player at {}, {}", spawn_pos.x, spawn_pos.y);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.player.clone(),
            // current sprite is 16x28
            transform: Transform {
                translation: Vec3::new(spawn_pos.x, spawn_pos.y, 1.),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE * 0.57, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(0.))
        .insert(Gravity(10.))
        .insert(Player::default())
        .insert(PlayerReadyAttack(true))
        .insert(Speed::default());

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
    mut query: Query<(&Speed, &mut Player, &mut Transform, With<Player>)>,
) {
    if let Ok((speed, mut player, mut player_tf, _)) = query.single_mut() {
        let move_x = if kb.pressed(KeyCode::Left) || kb.pressed(KeyCode::A) {
            -1.0
        } else if kb.pressed(KeyCode::Right) || kb.pressed(KeyCode::D) {
            1.0
        } else {
            0.
        };

        let new_x = player_tf.translation.x + move_x * speed.0 * TIME_STEP;
        if new_x < win_size.w / 2. && new_x > -win_size.w / 2. {
            player_tf.translation.x = new_x;
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
    mut query: Query<(Entity, &Velocity, With<Gravity>)>,
) {
    if let Ok((player_entity, velocity, _)) = query.single_mut() {
        if velocity.0 > -0.01
            && velocity.0 < 0.01
            && (kb.pressed(KeyCode::Up) || kb.pressed(KeyCode::W) || kb.pressed(KeyCode::Space))
        {
            commands.entity(player_entity).insert(Velocity(500.0));
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
                .insert(Speed(1000.));
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
        &mut Transform,
        With<Projectile>,
    )>,
) {
    for (proj_entity, projectile, proj_speed, mut proj_tf, _) in query.iter_mut() {
        let translation = &mut proj_tf.translation;
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
