mod components;
mod constants;
mod map;
mod physics;
mod player;
mod resources;

use bevy::prelude::*;
use map::MapPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use resources::{CharacterTileset, Materials, Tilesets, WinSize};

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Hackweek 2021".to_string(),
            width: 1400.0,
            height: 768.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // position window
    let window = windows.get_primary_mut().unwrap();
    window.set_position(IVec2::new(2000, 0));

    // create the main resources
    commands.insert_resource(Materials {
        player: materials.add(asset_server.load("knight_f_idle_anim_f0.png").into()),
        projectile: materials.add(asset_server.load("new/bullet.png").into()),

        bg_forest: materials.add(asset_server.load("new/background1.png").into()),
        bg_snow: materials.add(asset_server.load("new/background2.png").into()),

        background: materials.add(asset_server.load("new2/Background.png").into()),
        tile_left: materials.add(asset_server.load("new2/TileSet_01.png").into()),
        tile_middle: materials.add(asset_server.load("new2/TileSet_02.png").into()),
        tile_right: materials.add(asset_server.load("new2/TileSet_03.png").into()),
        tile_island: materials.add(asset_server.load("new2/TileSet_04.png").into()),
        tile_spikes: materials.add(asset_server.load("new2/Spikes1_1.png").into()),
    });

    commands.insert_resource(CharacterTileset {
        hero: texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("new2/PrototypeHero_noSword.png").into(),
            Vec2::new(100.0, 100.0),
            8,
            12,
        )),
    });

    commands.insert_resource(Tilesets {
        forest: texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("new/forest.png").into(),
            Vec2::new(16.0, 16.0),
            12,
            6,
        )),
        snow: texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("new/snow.png").into(),
            Vec2::new(16.0, 16.0),
            12,
            6,
        )),

        spikes: texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load("new/spikes.png").into(),
            Vec2::new(16.0, 16.0),
            4,
            1,
        )),
    });

    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });
}
