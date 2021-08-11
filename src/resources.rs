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
    pub tile_wall_left: Handle<ColorMaterial>,
    pub tile_wall_middle: Handle<ColorMaterial>,
    pub tile_wall_right: Handle<ColorMaterial>,
    pub tile_edge: Handle<ColorMaterial>,
    pub tile_spike: Handle<ColorMaterial>,
}

pub struct WinSize {
    #[allow(unused)]
    pub w: f32,
    pub h: f32,
}
