use std::path::*;

use bevy::{prelude::*, input::{*, mouse::*, keyboard::keyboard_input_system}};
use serde::*;

#[derive(Default)]
pub struct BlackboxPlugin {
    file_name: Option<String>,
}

impl BlackboxPlugin {
    pub fn record() -> Self {
        Self { file_name: None }
    }
    
    pub fn play(file_name: impl ToString) -> Self {
        Self { file_name: Some(file_name.to_string()) }
    }
}

impl Plugin for BlackboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlaybackState>();
            
        if let Some(file_name) = &self.file_name {
            app.add_systems(PreUpdate, 
                (
                    advance_playback,
                    pass_played_inputs,
                ).chain()
                    .in_set(InputSystem)
                    .after(mouse_button_input_system)
                    .after(keyboard_input_system)
                    .run_if(state_exists_and_equals(PlaybackState::Playing))
            )
            .add_systems(Startup, (
                 |mut ps: ResMut<NextState<PlaybackState>>| ps.set(PlaybackState::Playing),
             ))
            .insert_resource(make_input_playback_context(&file_name));
        } else {
            app.add_systems(PreUpdate, (
                record_inputs.after(InputSystem),
            ))
            .insert_resource(InputRecordContext {
                last_record: None,
                file_path: new_file(),
                update_count: 0,
            });
        }
    }
}

const SAVE_DIRECTORY: &'static str = "input_records";

fn new_file() -> PathBuf {
    std::fs::create_dir_all(SAVE_DIRECTORY)
        .expect("unable to create input records folder");

    let file_id = instant::now() as u64;
    let file_name = format!("{file_id}.record.ron");
    let path_string = format!("{SAVE_DIRECTORY}/{file_name}");
    let path = PathBuf::from(path_string);

    std::fs::File::create(&path)
        .expect("unable to create input records file");

    path
}

#[derive(Default, Resource, Serialize, Deserialize)]
struct InputRecord {
    #[serde(rename = "e")] elapsed_seconds: f64,
    #[serde(rename = "u")] update_index: u64,
    #[serde(rename = "i")] inner: InputRecordInner,
}

#[derive(Default, PartialEq, Serialize, Deserialize)]
struct InputRecordInner {
    #[serde(rename = "k")] keys: PressedAndReleased<KeyCode>,
    #[serde(rename = "m")] mouse_buttons: PressedAndReleased<MouseButton>,
    #[serde(rename = "o")] mouse_motion: Vec2,
    #[serde(rename = "c")] cursor_pos: Vec2,
}

#[derive(PartialEq, Serialize, Deserialize)]
struct PressedAndReleased<T> {
    #[serde(rename = "p")] pressed: Vec<T>,
    #[serde(rename = "r")] released: Vec<T>,
}

impl<T> Default for PressedAndReleased<T> {
    fn default() -> Self {
        Self { pressed: Vec::new(), released: Vec::new() }
    }
}

fn record_pressed_and_relased<T>(input: &Input<T>) -> PressedAndReleased<T> 
  where T: Copy + Eq + std::hash::Hash + Send + Sync + 'static {
    PressedAndReleased {
        pressed: input.get_just_pressed().copied().collect(),
        released: input.get_just_released().copied().collect(),
    }
}

fn playback_pressed_and_released<T>(par: &PressedAndReleased<T>, input: &mut Input<T>) 
  where T: Copy + Eq + std::hash::Hash + Send + Sync + 'static {
    for pressed in &par.pressed {
        input.press(*pressed);
    }
    for pressed in &par.released {
        input.release(*pressed);
    }
}

#[derive(Resource)]
struct InputRecordContext {
    last_record: Option<InputRecord>,
    file_path: PathBuf,
    update_count: u64,
}

fn record_inputs(
    time: Res<Time>,
    mut ctx: ResMut<InputRecordContext>, 
    keyboard: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    window: Query<&Window>,
) {
    use std::{fs::*, io::*};

    let record = InputRecordInner {
        keys: record_pressed_and_relased(&*keyboard),
        cursor_pos: window.single().cursor_position().unwrap_or(Vec2::NEG_ONE),
        mouse_buttons: record_pressed_and_relased(&*mouse_buttons),
        mouse_motion: mouse_motion.read().map(|m| m.delta).sum(),
    };

    if ctx.last_record.as_ref().map(|last_record| last_record.inner != record).unwrap_or(true) {
        let record = InputRecord {
            elapsed_seconds: time.elapsed_seconds_f64(),
            update_index: ctx.update_count,
            inner: record,
        };

        let serialized_inputs = ron::ser::to_string(&record)
            .expect("unable to serialize world");

        let mut file = OpenOptions::new()
            .append(true)
            .open(&ctx.file_path)
            .expect("unable to open input records file");

        write!(file, "{}\n", serialized_inputs).expect("unable to write to input records file!");

        ctx.last_record = Some(record);
    }

    ctx.update_count += 1;
}

#[derive(Resource)]
struct InputPlaybackContext {
    current_record: InputRecord,
    next_record: InputRecord,
    remaining: String,
    just_updated: bool,
}

const DESER_FAIL: &'static str = "could not deserialize input record";

fn next_record(lines: &mut String) -> Option<InputRecord> {
    if lines.len() <= 2 { return None }

    let endln_pos = lines.chars().position(|c| c == '\n').unwrap_or(lines.len() - 1);
    let line = lines.drain(0..=endln_pos).collect::<String>();

    Some(ron::de::from_str(&*line).expect(DESER_FAIL))
}

fn make_input_playback_context(file: &String) -> InputPlaybackContext {
    let mut playback_file = {
        use std::{fs::*, io::*};

        let possible_file_paths = [
            format!("{SAVE_DIRECTORY}/{file}.record.ron"),
            format!("{SAVE_DIRECTORY}/{file}.ron"),
            format!("{SAVE_DIRECTORY}/{file}"),
        ];

        let file = possible_file_paths.into_iter().find_map(|path| {
            info!("trying to open: {path}");
            File::open(path).ok()
        }).expect("searched through all possible paths, could not find input record");

        let mut string = String::new();
        { file }.read_to_string(&mut string).expect("could not read input record");
        string
    };

    let current_record = next_record(&mut playback_file).unwrap();
    let next_record = next_record(&mut playback_file).unwrap();

    InputPlaybackContext { 
        current_record, 
        next_record, 
        remaining: playback_file,
        just_updated: false,
    }
}

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
enum PlaybackState {
    Playing,
    #[default]
    NotPlaying,
}

fn advance_playback(
    time: Res<Time>, 
    mut playback_ctx: ResMut<InputPlaybackContext>,
    mut playback_state: ResMut<NextState<PlaybackState>>,
) {
    if playback_ctx.next_record.elapsed_seconds <= time.elapsed_seconds_f64() {
        if let Some(next_record) = next_record(&mut playback_ctx.remaining) {
            playback_ctx.current_record =
                std::mem::replace(&mut playback_ctx.next_record, next_record);
        } else {
            playback_ctx.current_record =
                std::mem::replace(&mut playback_ctx.next_record, InputRecord::default());
            playback_state.set(PlaybackState::NotPlaying);
            info!("done playing back inputs.");
        }

        playback_ctx.just_updated = true;
    } else {
        playback_ctx.just_updated = false;
    }
}

fn pass_played_inputs(
    record: Res<InputPlaybackContext>, 
    mut keyboard: ResMut<Input<KeyCode>>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
    mut mouse_motion: EventWriter<MouseMotion>,
    mut window: Query<&mut Window>,
) {
    let inner_record = &record.current_record.inner;

    if record.just_updated {
        playback_pressed_and_released(&inner_record.keys, &mut keyboard);
        playback_pressed_and_released(&inner_record.mouse_buttons, &mut mouse_buttons);
        mouse_motion.send(MouseMotion { delta: inner_record.mouse_motion });
    }

    window.single_mut().set_cursor_position(Some(inner_record.cursor_pos));
}
