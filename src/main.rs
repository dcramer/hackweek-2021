mod components;
mod constants;
mod map;
mod physics;
mod player;
mod resources;

use bevy::prelude::*;
use map::MapPlugin;
use physics::{DebugPhysicsPlugin, PhysicsPlugin};
use player::PlayerPlugin;
use resources::{CharacterAnimation, Materials, WinSize};

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
        .add_plugin(PhysicsPlugin)
        // .add_plugin(DebugPhysicsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // position window
    let window = windows.get_primary_mut().unwrap();
    window.set_position(IVec2::new(2000, 0));

    commands.insert_resource(CharacterAnimation {
        idle_f0: materials.add(
            asset_server
                .load("anim/idle/knight_m_idle_anim_f0.png")
                .into(),
        ),
        idle_f1: materials.add(
            asset_server
                .load("anim/idle/knight_m_idle_anim_f1.png")
                .into(),
        ),
        idle_f2: materials.add(
            asset_server
                .load("anim/idle/knight_m_idle_anim_f2.png")
                .into(),
        ),
        idle_f3: materials.add(
            asset_server
                .load("anim/idle/knight_m_idle_anim_f3.png")
                .into(),
        ),

        run_f0: materials.add(
            asset_server
                .load("anim/run/knight_m_run_anim_f0.png")
                .into(),
        ),
        run_f1: materials.add(
            asset_server
                .load("anim/run/knight_m_run_anim_f1.png")
                .into(),
        ),
        run_f2: materials.add(
            asset_server
                .load("anim/run/knight_m_run_anim_f2.png")
                .into(),
        ),
        run_f3: materials.add(
            asset_server
                .load("anim/run/knight_m_run_anim_f3.png")
                .into(),
        ),
    });

    // create the main resources
    commands.insert_resource(Materials {
        red: materials.add(Color::rgb(1., 0., 0.).into()),
        green: materials.add(Color::rgb(0., 1., 0.).into()),
        blue: materials.add(Color::rgb(0., 0., 1.).into()),

        projectile: materials.add(asset_server.load("bullet.png").into()),

        tile_wall_left: materials.add(asset_server.load("wall_left.png").into()),
        tile_wall_middle: materials.add(asset_server.load("wall_mid.png").into()),
        tile_wall_right: materials.add(asset_server.load("wall_right.png").into()),
        tile_edge: materials.add(asset_server.load("edge.png").into()),
        tile_ladder: materials.add(asset_server.load("ladder.png").into()),

        tile_lava_01: materials.add(asset_server.load("lava_01.png").into()),
        tile_lava_02: materials.add(asset_server.load("lava_02.png").into()),
        tile_lava_03: materials.add(asset_server.load("lava_03.png").into()),
        tile_lava_04: materials.add(asset_server.load("lava_04.png").into()),
        tile_lava_05: materials.add(asset_server.load("lava_05.png").into()),
        tile_lava_06: materials.add(asset_server.load("lava_06.png").into()),
        tile_lava_07: materials.add(asset_server.load("lava_07.png").into()),
        tile_lava_08: materials.add(asset_server.load("lava_08.png").into()),
        tile_lava_09: materials.add(asset_server.load("lava_09.png").into()),
        tile_lava_10: materials.add(asset_server.load("lava_10.png").into()),
        tile_lava_11: materials.add(asset_server.load("lava_11.png").into()),
        tile_lava_12: materials.add(asset_server.load("lava_12.png").into()),
    });

    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });
}
