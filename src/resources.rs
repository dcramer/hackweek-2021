use bevy::prelude::*;

pub struct CharacterAnimation {
    pub idle_f0: Handle<ColorMaterial>,
    pub idle_f1: Handle<ColorMaterial>,
    pub idle_f2: Handle<ColorMaterial>,
    pub idle_f3: Handle<ColorMaterial>,

    pub run_f0: Handle<ColorMaterial>,
    pub run_f1: Handle<ColorMaterial>,
    pub run_f2: Handle<ColorMaterial>,
    pub run_f3: Handle<ColorMaterial>,
}

pub struct Materials {
    pub projectile: Handle<ColorMaterial>,

    pub background: Handle<ColorMaterial>,
    pub tile_left: Handle<ColorMaterial>,
    pub tile_middle: Handle<ColorMaterial>,
    pub tile_right: Handle<ColorMaterial>,
    pub tile_island: Handle<ColorMaterial>,
    pub tile_spikes: Handle<ColorMaterial>,
    pub tile_platform_left: Handle<ColorMaterial>,
    pub tile_platform_middle: Handle<ColorMaterial>,
    pub tile_platform_right: Handle<ColorMaterial>,
    pub tile_platform_island: Handle<ColorMaterial>,

    pub tile_wall_left: Handle<ColorMaterial>,
    pub tile_wall_middle: Handle<ColorMaterial>,
    pub tile_wall_right: Handle<ColorMaterial>,
    pub tile_edge: Handle<ColorMaterial>,
}

pub struct Tilesets {
    pub forest: Handle<TextureAtlas>,
    pub snow: Handle<TextureAtlas>,
    pub spikes: Handle<TextureAtlas>,
}

pub struct CharacterTileset {
    pub hero: Handle<TextureAtlas>,
}

pub struct WinSize {
    #[allow(unused)]
    pub w: f32,
    pub h: f32,
}
