use bevy::prelude::*;

pub struct Materials {
    pub player: Handle<ColorMaterial>,
    pub projectile: Handle<ColorMaterial>,

    pub bg_forest: Handle<ColorMaterial>,
    pub bg_snow: Handle<ColorMaterial>,
}

pub struct Tilesets {
    pub forest: Handle<TextureAtlas>,
    pub snow: Handle<TextureAtlas>,
    pub spikes: Handle<TextureAtlas>,
}

pub struct WinSize {
    #[allow(unused)]
    pub w: f32,
    pub h: f32,
}
