use crate::{
    components::{Collider, Tile},
    resources::Materials,
};
use bevy::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Platform,
    Ladder,
    Solid,
    Lava,
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

fn map_render(mut commands: Commands, map: Res<Map>, materials: Res<Materials>) {
    let tile_size = map.tile_size as f32;
    let half_tile_size = tile_size / 2.0;
    for (i, tile) in map
        .tiles
        .iter()
        .enumerate()
        .filter(|x| *x.1 != TileType::Empty)
    {
        let pos = map.tile_position(i as i32 % map.width, i as i32 / map.width);
        let mut sprite_scale = 2.0;
        let mut collider_center = Vec2::new(pos.x + half_tile_size, pos.y + half_tile_size);
        let mut collider_half = Vec2::new(half_tile_size, half_tile_size);
        let material = match &tile {
            TileType::Lava => {
                sprite_scale = 1.0;
                Some(materials.tile_lava_01.clone())
            }
            TileType::Ladder => Some(materials.tile_ladder.clone()),
            TileType::Empty => None,
            TileType::Platform => {
                collider_center = Vec2::new(pos.x + half_tile_size, pos.y + 6.);
                collider_half = Vec2::new(half_tile_size, 6.);

                // previous tile was same row and a solid?
                if (i - 1) / map.width as usize == i / map.width as usize
                    && map.tiles[i - 1] == TileType::Platform
                {
                    // current tile is not end of row or next tile is solid
                    if i as i32 % map.width != map.width - 1 as i32
                        && map.tiles[i + 1] == TileType::Platform
                    {
                        Some(materials.tile_edge.clone())
                    } else {
                        Some(materials.tile_edge.clone())
                    }
                // previous tile was air or first tile in row
                } else if (i - 1) / map.width as usize != i / map.width as usize
                    || map.tiles[i - 1] != TileType::Platform
                {
                    // current tile is end of row
                    if i as i32 % map.width == map.width - 1 as i32 {
                        Some(materials.tile_edge.clone())
                    // next tile is air
                    } else if map.tiles[i + 1] != TileType::Platform {
                        Some(materials.tile_edge.clone())
                    } else {
                        Some(materials.tile_edge.clone())
                    }
                } else {
                    Some(materials.tile_edge.clone())
                }
            }
            TileType::Solid => {
                // previous tile was same row and a solid?
                if (i - 1) / map.width as usize == i / map.width as usize
                    && map.tiles[i - 1] == TileType::Solid
                {
                    // current tile is not end of row or next tile is solid
                    if i as i32 % map.width != map.width - 1 as i32
                        && map.tiles[i + 1] == TileType::Solid
                    {
                        Some(materials.tile_wall_middle.clone())
                    } else {
                        Some(materials.tile_wall_right.clone())
                    }
                // previous tile was air or first tile in row
                } else if (i - 1) / 32 != i / 32 || map.tiles[i - 1] != TileType::Solid {
                    // current tile is end of row
                    if i as i32 % map.width == map.width - 1 as i32 {
                        Some(materials.tile_wall_middle.clone())
                    // next tile is air
                    } else if map.tiles[i + 1] != TileType::Solid {
                        Some(materials.tile_wall_middle.clone())
                    } else {
                        Some(materials.tile_wall_left.clone())
                    }
                } else {
                    Some(materials.tile_wall_middle.clone())
                }
            }
        };
        commands
            .spawn_bundle(SpriteBundle {
                material: material.unwrap(),
                transform: Transform {
                    translation: Vec3::new(pos.x, pos.y, 1.),
                    scale: Vec3::new(sprite_scale, sprite_scale, 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Collider::new(collider_center, collider_half))
            .insert(Tile);
    }
}

pub struct Map {
    pub position: Vec3,

    // width and height in tiles
    pub width: i32,
    pub height: i32,
    pub tile_size: i32,
    pub tiles: Vec<TileType>,
    pub starting_positions: Vec<Vec2>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            width: 0,
            height: 0,
            tiles: vec![TileType::Empty; 0 as usize],
            tile_size: 32,
            starting_positions: vec![Vec2::ZERO; 4],
        }
    }
}

// TODO: abstract tiles into a TileMap
impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Empty; (width * height) as usize],
            ..Default::default()
        }
    }

    pub fn from_position(position: Vec3, width: i32, height: i32) -> Self {
        Self {
            position,
            width,
            height,
            tiles: vec![TileType::Empty; (width * height) as usize],
            ..Default::default()
        }
    }

    pub fn from_prefab(prefab: (&str, i32, i32)) -> Self {
        let mut map = Self::from_position(
            // center the map relative to 0/0
            Vec3::new(
                0.5 - prefab.1 as f32 * 32.0 / 2.,
                0.5 - prefab.2 as f32 * 32.0 / 2.,
                1.,
            ),
            prefab.1,
            prefab.2,
        );
        let mut new_tiles = map.tiles.clone();
        let mut starting_positions = Vec::new();

        let string_vec: Vec<char> = prefab
            .0
            .chars()
            .filter(|a| *a != '\r' && *a != '\n' && *a != ' ')
            .collect();
        let mut i = 0;
        for ty in 0..prefab.2 {
            for tx in 0..prefab.1 {
                let idx = map.tile_index(tx, map.height - ty - 1);
                let c = string_vec[i];
                match c {
                    '-' => new_tiles[idx] = TileType::Empty,
                    '^' => new_tiles[idx] = TileType::Lava,
                    '=' => new_tiles[idx] = TileType::Platform,
                    '|' => new_tiles[idx] = TileType::Ladder,
                    'X' => {
                        starting_positions.push(map.tile_position(tx, map.height - ty - 1));
                        new_tiles[idx] = TileType::Empty
                    }
                    '#' => new_tiles[idx] = TileType::Solid,
                    _ => println!("No idea what to do with [{}]", c),
                }
                i += 1;
            }
        }

        map.starting_positions = starting_positions;
        map.tiles = new_tiles;
        map
    }

    /// returns the the tile located at map position
    pub fn tile_at_point(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            self.tile_x_at_point(point.x) as f32,
            self.tile_y_at_point(point.y) as f32,
        )
    }

    pub fn tile_x_at_point(&self, x: f32) -> i32 {
        (x - self.position.x).ceil() as i32 / self.tile_size
    }

    pub fn tile_y_at_point(&self, y: f32) -> i32 {
        (y - self.position.y).ceil() as i32 / self.tile_size
    }

    /// translate relative tile position to map position
    pub fn tile_position(&self, tile_x: i32, tile_y: i32) -> Vec2 {
        Vec2::new(
            ((tile_x * self.tile_size) as f32 + self.position.x).ceil(),
            ((tile_y * self.tile_size) as f32 + self.position.y).ceil(),
        )
    }

    /// translate relative tile position to index in tilemap
    pub fn tile_index(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }

    // return the tile at relative position
    pub fn tile(&self, x: i32, y: i32) -> TileType {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            TileType::Solid
        } else {
            self.tiles[self.tile_index(x, y)]
        }
    }

    pub fn is_obstacle(&self, x: i32, y: i32) -> bool {
        let tile = self.tile(x, y);
        tile == TileType::Solid || tile == TileType::Lava
    }

    pub fn is_platform(&self, x: i32, y: i32) -> bool {
        let tile = self.tile(x, y);
        tile == TileType::Platform
    }

    pub fn is_ladder(&self, x: i32, y: i32) -> bool {
        let tile = self.tile(x, y);
        tile == TileType::Ladder
    }

    pub fn is_ground(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            self.tiles[((y * self.width) + x) as usize] == TileType::Solid
        }
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        let tile = self.tile(x, y);
        tile == TileType::Empty
    }
}

// position is bottom left corner
const DEFAULT_MAP: (&str, i32, i32) = (
    "
------------------------------------------------
------------------------------------------------
------------------------------------------------
--------------X---------------------------------
-------------#####------------------------------
---------------------====-----X---==------------
-----------------------------###----------------
---------------#----##----#|------==------------
---------------#----------#|--------------------
---------------####====####|---#--==------------
--------------X------------|---#--X-------------
----------###########################-----------
------------------------------------------------
----===---------------------------------===-----
------------------------------------------------
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
",
    48,
    16,
);

pub fn build_default_map() -> Map {
    Map::from_prefab(DEFAULT_MAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() {
        let prefab: (&str, i32, i32) = (
            "
            -X--
            -##-
            ^^^^
            ",
            4,
            3,
        );
        let mut map = Map::from_prefab(prefab);
        map.tile_size = 16;
        map.position = Vec3::ZERO;

        assert_eq!(map.tile(0, 0), TileType::Lava);
        assert_eq!(map.tile(1, 0), TileType::Lava);
        assert_eq!(map.tile(1, 1), TileType::Solid);
        assert_eq!(map.tile(1, 2), TileType::Empty);

        assert_eq!(map.is_obstacle(0, 0), true);
        assert_eq!(map.is_obstacle(1, 1), true);
        assert_eq!(map.is_obstacle(2, 1), true);
        assert_eq!(map.is_obstacle(0, 1), false);
        assert_eq!(map.is_obstacle(2, 2), false);

        assert_eq!(map.tile_position(0, 0), Vec2::new(0., 0.));
        assert_eq!(map.tile_position(3, 2), Vec2::new(48., 32.));

        assert_eq!(map.tile_x_at_point(48.), 3);
        assert_eq!(map.tile_y_at_point(32.), 2);
    }

    #[test]
    fn tile_index() {
        let mut map = Map::new(32, 16);
        map.tile_size = 16;
        assert_eq!(map.tile_index(0, 0), 0);
        assert_eq!(map.tile_index(3, 15), 483);
        assert_eq!(map.tile_index(35, 13), 451);
    }

    #[test]
    fn tile_position() {
        let mut map = Map::new(32, 16);
        map.tile_size = 16;
        map.position = Vec3::new(-256., -128., 0.);
        assert_eq!(map.tile_position(31, 15), Vec2::new(240., 112.));
        assert_eq!(map.tile_position(0, 0), Vec2::new(-256., -128.));
    }

    #[test]
    fn tile_at_point() {
        let mut map = Map::new(32, 16);
        map.tile_size = 16;
        map.position = Vec3::new(-256., -128., 0.);
        assert_eq!(
            map.tile_at_point(Vec2::new(244., 116.)),
            Vec2::new(31., 15.)
        );
        assert_eq!(map.tile_at_point(Vec2::new(-159., 65.)), Vec2::new(6., 12.));
        assert_eq!(
            map.tile_at_point(Vec2::new(-256., -128.)),
            Vec2::new(0., 0.)
        );
    }
}
