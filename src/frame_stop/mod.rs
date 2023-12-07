use std::time::Duration;

use bevy::input::InputSystem;

use crate::prelude::*;

pub struct FrameStopPlugin;

impl Plugin for FrameStopPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, frame_stop.after(InputSystem))
            .init_resource::<FrameStopState>();
    }
}

#[derive(Default, Resource, PartialEq)]
enum FrameStopState { Stopped, #[default] Continue }

fn frame_stop(
    keys: Res<Input<KeyCode>>, 
    mut time: ResMut<Time>,
    mut timev: ResMut<Time<Virtual>>,
    mut timef: ResMut<Time<Fixed>>,
    mut stop_state: ResMut<FrameStopState>
) {
    let control_pressed = keys.pressed(crate::CONTROL_KEY);

    if *stop_state == FrameStopState::Stopped {
        let pre_time = *time;
        let pre_timev = *timev;
        let pre_timef = *timef;
        std::thread::sleep(Duration::from_millis(1000));
        *time = pre_time;
        *timev = pre_timev;
        *timef = pre_timef;

        if control_pressed && keys.pressed(KeyCode::ShiftLeft) && keys.pressed(KeyCode::F) {
            *stop_state = FrameStopState::Continue; 
        }
    } else {
        if control_pressed && keys.just_pressed(KeyCode::F) {
            *stop_state = FrameStopState::Stopped; 
        }
    }

}
