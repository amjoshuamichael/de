use std::time::*;

use bevy::prelude::*;

#[derive(Default, Component)]
pub struct TimedInteraction {
    pub double_clicked: bool,
    clicked: bool,
    pub single_clicked: bool,
}

const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(300);

pub fn do_timed_interactions(
    mouse: Res<Input<MouseButton>>,
    mut last_click: Local<Option<Instant>>,
    mut timed_interactions: Query<(&mut TimedInteraction, &Interaction)>,
) {
    for mut timed_interaction in &mut timed_interactions {
        timed_interaction.0.double_clicked = false;
        timed_interaction.0.single_clicked = false;
    }

    if mouse.just_pressed(MouseButton::Left) {
        let now = Instant::now();

        if let Some(last) = *last_click && now.duration_since(last) < DOUBLE_CLICK_TIME {
            for mut timed_interaction in &mut timed_interactions {
                if *timed_interaction.1 == Interaction::Pressed {
                    timed_interaction.0.double_clicked = true;
                }
            }

            *last_click = None;
        } else {
            for mut timed_interaction in &mut timed_interactions {
                if *timed_interaction.1 == Interaction::Pressed {
                    timed_interaction.0.clicked = true;
                }
            }

            *last_click = Some(now);
        }
    } else if let Some(last) = *last_click && last.elapsed() > DOUBLE_CLICK_TIME {
        if mouse.pressed(MouseButton::Left) {
            for mut timed_interaction in &mut timed_interactions {
                timed_interaction.0.clicked = false;
            }
        } else {
            for mut timed_interaction in &mut timed_interactions {
                if timed_interaction.0.clicked {
                    timed_interaction.0.single_clicked = true;
                }

                timed_interaction.0.clicked = false;
            }
        }

        *last_click = None;
    }
}
