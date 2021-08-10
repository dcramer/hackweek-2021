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
use resources::{CharacterAnimation, CharacterTileset, Materials, Tilesets, WinSize};

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

    commands.insert_resource(CharacterAnimation {
        idle_f0: materials.add(
            asset_server
                .load("new3/anim/idle/knight_m_idle_anim_f0.png")
                .into(),
        ),
        idle_f1: materials.add(
            asset_server
                .load("new3/anim/idle/knight_m_idle_anim_f1.png")
                .into(),
        ),
        idle_f2: materials.add(
            asset_server
                .load("new3/anim/idle/knight_m_idle_anim_f2.png")
                .into(),
        ),
        idle_f3: materials.add(
            asset_server
                .load("new3/anim/idle/knight_m_idle_anim_f3.png")
                .into(),
        ),

        run_f0: materials.add(
            asset_server
                .load("new3/anim/run/knight_m_run_anim_f0.png")
                .into(),
        ),
        run_f1: materials.add(
            asset_server
                .load("new3/anim/run/knight_m_run_anim_f1.png")
                .into(),
        ),
        run_f2: materials.add(
            asset_server
                .load("new3/anim/run/knight_m_run_anim_f2.png")
                .into(),
        ),
        run_f3: materials.add(
            asset_server
                .load("new3/anim/run/knight_m_run_anim_f3.png")
                .into(),
        ),
    });

    // create the main resources
    commands.insert_resource(Materials {
        projectile: materials.add(asset_server.load("new/bullet.png").into()),

        background: materials.add(asset_server.load("new2/Background.png").into()),
        tile_left: materials.add(asset_server.load("new2/TileSet_01.png").into()),
        tile_middle: materials.add(asset_server.load("new2/TileSet_02.png").into()),
        tile_right: materials.add(asset_server.load("new2/TileSet_03.png").into()),
        tile_island: materials.add(asset_server.load("new2/TileSet_04.png").into()),
        tile_spikes: materials.add(asset_server.load("new2/Spikes1_1.png").into()),
        tile_platform_left: materials.add(asset_server.load("new2/TileSet_36.png").into()),
        tile_platform_middle: materials.add(asset_server.load("new2/TileSet_37.png").into()),
        tile_platform_right: materials.add(asset_server.load("new2/TileSet_38.png").into()),
        tile_platform_island: materials.add(asset_server.load("new2/TileSet_21.png").into()),
        tile_wall_left: materials.add(asset_server.load("new3/wall_left.png").into()),
        tile_wall_middle: materials.add(asset_server.load("new3/wall_mid.png").into()),
        tile_wall_right: materials.add(asset_server.load("new3/wall_right.png").into()),
        tile_edge: materials.add(asset_server.load("new3/edge.png").into()),
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
