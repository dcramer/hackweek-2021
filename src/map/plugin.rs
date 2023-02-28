use bevy::prelude::*;

use crate::components::{Collider, Tile};

use super::{
    events::tile_collision_listener,
    generator::{build_default_map, TileType},
    Map,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let map = build_default_map();
        app.insert_resource(map)
            .add_startup_stage("map render", SystemStage::single(map_render))
            .add_system(tile_collision_listener);
    }
}

fn map_render(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    for (i, tile) in map
        .tiles
        .iter()
        .enumerate()
        .filter(|x| *x.1 != TileType::Empty)
    {
        let mut tile_size = Vec2::new(32., 32.);
        let mut half_tile_size = tile_size / 2.0;
        let pos = map.tile_position(i as i32 % map.width, i as i32 / map.width);

        let mut collider: Option<Collider> = Some(Collider::from_position(
            Vec3::from((pos, 0.)),
            half_tile_size,
        ));
        let mut sprite_scale = 2.0;

        let texture = match &tile {
            TileType::Lava => {
                sprite_scale = 1.0;
                Some(asset_server.load("lava_01.png"))
            }
            TileType::Ladder => {
                collider = None;
                Some(asset_server.load("ladder.png"))
            }
            TileType::Empty => None,
            TileType::Platform => {
                tile_size = Vec2::new(32., 12.);
                half_tile_size = tile_size / 2.;
                collider = Some(Collider {
                    pos,
                    half: Vec2::new(half_tile_size.x, 0.5),
                    top: true,
                    left: true,
                    right: true,
                    bottom: false,
                });
                Some(asset_server.load("edge.png"))
            }
            TileType::Solid => {
                collider = Some(Collider::from_position(
                    Vec3::from((pos, 0.)),
                    half_tile_size,
                ));
                // previous tile was same row and a solid?
                if (i - 1) / map.width as usize == i / map.width as usize
                    && map.tiles[i - 1] == TileType::Solid
                {
                    // current tile is not end of row or next tile is solid
                    if i as i32 % map.width != map.width - 1 as i32
                        && map.tiles[i + 1] == TileType::Solid
                    {
                        Some(asset_server.load("wall_mid.png"))
                    } else {
                        Some(asset_server.load("wall_right.png"))
                    }
                // previous tile was air or first tile in row
                } else if (i - 1) / 32 != i / 32 || map.tiles[i - 1] != TileType::Solid {
                    // current tile is end of row
                    if i as i32 % map.width == map.width - 1 as i32 {
                        Some(asset_server.load("wall_mid.png"))
                    // next tile is air
                    } else if map.tiles[i + 1] != TileType::Solid {
                        Some(asset_server.load("wall_mid.png"))
                    } else {
                        Some(asset_server.load("wall_left.png"))
                    }
                } else {
                    Some(asset_server.load("wall_mid.png"))
                }
            }
        };

        let mut entity = commands.spawn(SpriteBundle {
            texture: texture.unwrap(),
            transform: Transform {
                translation: Vec3::new(pos.x, pos.y, 1.),
                scale: Vec3::new(sprite_scale, sprite_scale, 1.),
                ..default()
            },
            ..default()
        });

        entity.insert(Tile);

        if let Some(c) = collider {
            entity.insert(c);
        }
    }
}
