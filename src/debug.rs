use bevy::prelude::*;

use crate::{
    components::{Direction, Movable},
    map::Map,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(MouseLoc(Vec2::new(0.0, 0.0)))
            .add_system(mouse_movement_updating_system.system())
            .add_system(position_mouse_click_system.system());
    }
}

#[derive(Default)]
struct State {
    // Set up from example
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

struct MouseLoc(Vec2);

fn select_character(
    mut state: ResMut<State>,
    mouse_pos: ResMut<MouseLoc>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
) {
    for event in state
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        println!("event: {:?} position: {:?}", event, mouse_pos.0);
    }
}

fn mouse_movement_updating_system(
    mut mouse_pos: ResMut<MouseLoc>,
    mut state: ResMut<State>,
    cursor_moved_events: Res<Events<CursorMoved>>,
) {
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        mouse_pos.0 = event.position;
    }
}
