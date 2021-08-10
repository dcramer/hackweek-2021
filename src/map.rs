use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::fmt;

use crate::{
    components::Tile,
    constants::SPRITE_SCALE,
    resources::{Materials, Tilesets, WinSize},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TileType {
    Empty,
    Solid,
    Spike,
}

impl fmt::Display for TileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let map = build_default_map();
        app.insert_resource(map)
            .add_startup_stage("map render", SystemStage::single(map_render.system()));
    }
}

// x = -128 to 128 (left to right)
// y = 128 to -128 (top to bottom)

fn map_render(
    mut commands: Commands,
    map: Res<Map>,
    tilesets: Res<Tilesets>,
    materials: Res<Materials>,
    win_size: Res<WinSize>,
) {
    // 256x128
    let scale_y = win_size.h as f32 / 128.0;
    commands.spawn_bundle(SpriteBundle {
        material: materials.bg_forest.clone(),
        transform: Transform::from_scale(Vec3::new(scale_y, scale_y, 0.)),
        ..Default::default()
    });
    ();
    let island_tile = 20;
    let left_tile = 44;
    let middle_tile = 45;
    let right_tile = 46;

    for (i, tile) in map
        .tiles
        .iter()
        .enumerate()
        .filter(|x| *x.1 != TileType::Empty)
    {
        let (texture_atlas, sprite) = match &tile {
            TileType::Spike => (tilesets.spikes.clone(), TextureAtlasSprite::new(0)),
            TileType::Empty => (tilesets.forest.clone(), TextureAtlasSprite::new(0)),
            TileType::Solid => {
                // previous tile was same row and a solid?
                let tnum = if (i - 1) / 32 == i / 32 && map.tiles[i - 1] == TileType::Solid {
                    // current tile is not end of row or next tile is solid
                    if i as i32 % map.width != map.width - 1 as i32
                        && map.tiles[i + 1] == TileType::Solid
                    {
                        middle_tile
                    } else {
                        right_tile
                    }
                // previous tile was air or first tile in row
                } else if (i - 1) / 32 != i / 32 || map.tiles[i - 1] != TileType::Solid {
                    // current tile is end of row
                    if i as i32 % map.width == map.width - 1 as i32 {
                        island_tile
                    // next tile is air
                    } else if map.tiles[i + 1] != TileType::Solid {
                        island_tile
                    } else {
                        left_tile
                    }
                } else {
                    middle_tile
                };
                (tilesets.forest.clone(), TextureAtlasSprite::new(tnum))
            }
        };
        let pos = map.coords_to_pos(i as i32 % map.width, i as i32 / map.width);
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas,
                sprite,
                transform: Transform::from_scale(Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0)),
                ..Default::default()
            })
            .insert(Tile)
            .insert_bundle(ColliderBundle {
                shape: ColliderShape::cuboid(SPRITE_SCALE / 2., SPRITE_SCALE / 2.),
                position: Vec2::new(pos.x * SPRITE_SCALE, pos.y * SPRITE_SCALE).into(),
                material: ColliderMaterial {
                    restitution: 0.,
                    friction: 0.,
                    ..Default::default()
                },
                mass_properties: ColliderMassProps::Density(2.0),
                ..Default::default()
            })
            .insert(ColliderPositionSync::Discrete)
            .insert(ColliderDebugRender::with_id(1));
    }

    // draw bordering collider
    // - left
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(2., map.height as f32 * SPRITE_SCALE),
            position: Vec2::new(-(map.width as f32 + 2.) / 2. * SPRITE_SCALE, 0.).into(),
            ..Default::default()
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: tilesets.forest.clone(),
            sprite: TextureAtlasSprite::new(40),
            transform: Transform::from_scale(Vec3::new(1., (map.height) as f32 * SPRITE_SCALE, 1.)),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(5));

    // - right
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(1.0, (map.height) as f32 * SPRITE_SCALE),
            position: Vec2::new((map.width as f32 + 2.) / 2. * SPRITE_SCALE, 0.).into(),
            ..Default::default()
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: tilesets.forest.clone(),
            sprite: TextureAtlasSprite::new(40),
            transform: Transform::from_scale(Vec3::new(1., (map.height) as f32 * SPRITE_SCALE, 1.)),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(5));
    // - top
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid((map.width as f32 + 2.) * SPRITE_SCALE, 1.0),
            position: Vec2::new(0., (map.height) as f32 / 2. * SPRITE_SCALE).into(),
            ..Default::default()
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: tilesets.forest.clone(),
            sprite: TextureAtlasSprite::new(40),
            transform: Transform::from_scale(Vec3::new(
                (map.width as f32 + 3.) * SPRITE_SCALE,
                1.,
                1.,
            )),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(5));
    // - bottom
    commands
        .spawn_bundle(ColliderBundle {
            shape: ColliderShape::cuboid((map.width as f32 + 2.) * SPRITE_SCALE, 1.0),
            position: Vec2::new(0., -(map.height) as f32 / 2. * SPRITE_SCALE).into(),
            ..Default::default()
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: tilesets.forest.clone(),
            sprite: TextureAtlasSprite::new(40),
            transform: Transform::from_scale(Vec3::new(
                (map.width as f32 + 3.) * SPRITE_SCALE,
                1.,
                1.,
            )),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(5));
}

pub struct Map {
    width: i32,
    height: i32,
    tiles: Vec<TileType>,
    pub starting_positions: Vec<Vec2>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Empty; (width * height) as usize],
            starting_positions: vec![Vec2::ZERO; 4],
        }
    }

    pub fn coords_to_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }

    // convert to absolute x/y used by bevy
    pub fn coords_to_pos(&self, x: i32, y: i32) -> Vec2 {
        Vec2::new(
            (-(self.width / 2) + x) as f32 + 0.5,
            ((self.height / 2) - y) as f32 - 0.5,
        )
    }
}

const DFEAULT_MAP: (&str, i32, i32) = (
    "
--------------------------------
--------------------------------
-----#####----------------------
-----X----------------X---------
--##############-----###--------
--##############----------#-----
----------------------X---#-----
---------------------#########--
------X-----######--------------
--########----------------------
--------------------------------
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
",
    32,
    12,
);

pub fn build_default_map() -> Map {
    let mut map = Map::new(DFEAULT_MAP.1, DFEAULT_MAP.2);
    let mut new_tiles = map.tiles.clone();
    let mut starting_positions = Vec::new();

    let string_vec: Vec<char> = DFEAULT_MAP
        .0
        .chars()
        .filter(|a| *a != '\r' && *a != '\n')
        .collect();
    let mut i = 0;
    for ty in 0..DFEAULT_MAP.2 {
        for tx in 0..DFEAULT_MAP.1 {
            let idx = map.coords_to_idx(tx, ty);
            let c = string_vec[i];
            match c {
                '-' => new_tiles[idx] = TileType::Empty,
                '^' => new_tiles[idx] = TileType::Spike,
                'X' => {
                    starting_positions.push(map.coords_to_pos(tx, ty));
                    new_tiles[idx] = TileType::Empty
                }
                '#' => new_tiles[idx] = TileType::Solid,
                _ => println!("No idea what to do with [{}]", c),
            }
            i += 1;
        }
    }

    // for y in 0..map.height {
    //     for x in 0..map.width {
    //         if y % 3 == 1 && x > 3 && x < 28 && x % 2 == 1 {
    //             new_tiles[map.coords_to_idx(x, y)] = Tile::Solid;
    //         }
    //     }
    // }
    map.starting_positions = starting_positions;
    map.tiles = new_tiles;
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coords_to_idx() {
        let map = Map::new(32, 16);
        assert_eq!(map.coords_to_idx(3, 0), 3);
        assert_eq!(map.coords_to_idx(35, 2), 99);
    }

    #[test]
    fn coords_to_pos() {
        let map = Map::new(32, 16);
        assert_eq!(map.coords_to_pos(3, 0), Vec2::new(-208., 128.0));
        assert_eq!(map.coords_to_pos(35, 9), Vec2::new(304., -16.));
    }
}
