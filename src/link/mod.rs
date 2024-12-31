use crate::mischief::{MischiefEvent, MischiefEventData};
use crate::util::cleanup_system;
use crate::CurrentGame;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::StateInspectorPlugin;
use bevy_xpbd_2d::prelude::*;
use gameplay::GameplayPlugin;
use player::{AttachState, PlayerPlugin};
use spawn_level::{SpawnPlugin, SpawnState};

mod gameplay;
mod player;
mod spawn_level;

// MVP brief features:

// You control two ends of a physics-simulated rope with two mice.
// You use the rope to sort green circles and purple squares into two containers.
// 10 shapes will fall from above the screen over the course of the level.
// Your score is how many shapes you sorted correctly minus how many you sorted incorrectly.
// Shapes may also fall out the bottom of the screen, which doesn't penalize your score.
// The game ends when all 10 shapes have fallen.
// Click any mouse button to start a new game.

// MVP is in place! Polish time.

// Polish:
// Sound effects!
// Spawn shapes in more interesting ways. Randomized params, spawn in waves, spawn in patterns.
// Round the rest of the corners on the right side of the level.
// Visual polish on the level shapes.
// Add drop shadows to shapes and cursor/chain.
// Improve the game over screen layout.
// Add left and right mouse button images to the title/setup screen.

// Done polish:
// Differentiate left vs right cursors visually. (done)
// Pick a nicer color palette and recolor everything with it. (done)
// Add a title screen shown during AppState::Init. (done)
// Add game over screen shown during AppState::GameOver. (done)
// Increase intensity over time. (done)
// Two shape patterns (sequence and shotgun). (done)

// Bugs:
// - Window resolution doesn't seem to be working as I expect it to.

pub const LEFT_COLOR: Color = Color::rgb(17.0 / 255.0, 159.0 / 255.0, 166.0 / 255.0);
pub const RIGHT_COLOR: Color = Color::rgb(226.0 / 255.0, 101.0 / 255.0, 60.0 / 255.0);
pub const TEXT_COLOR: Color = Color::rgb(215.0 / 255.0, 217.0 / 255.0, 206.0 / 255.0);
pub const BAD_COLOR: Color = Color::rgb(229.0 / 255.0, 39.0 / 255.0, 36.0 / 255.0);

pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin)
            .add_plugins(SpawnPlugin)
            .add_plugins(GameplayPlugin)
            .add_plugins(PhysicsPlugins::new(FixedUpdate))
            .insert_resource(SubstepCount(20))
            .add_state::<AppState>()
            .add_plugins(StateInspectorPlugin::<AppState>::default().run_if(
                input_toggle_active(false, KeyCode::Grave).and_then(not(in_state(AppState::None))),
            ))
            .add_systems(Update, start_playing.run_if(in_state(AppState::Init)))
            .add_systems(OnExit(AppState::Init), cleanup_system::<DespawnOnExitInit>)
            .add_systems(Update, start_new_game.run_if(in_state(AppState::GameOver)))
            .add_systems(
                OnExit(AppState::GameOver),
                cleanup_system::<DespawnOnExitGameOver>,
            )
            .add_systems(OnEnter(CurrentGame::Link), on_enter_link)
            .add_systems(
                OnExit(CurrentGame::Link),
                (on_exit_link, cleanup_system::<DespawnOnExitLink>),
            );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum AppState {
    Init,
    Playing,
    GameOver,
    #[default]
    None,
}

fn start_playing(
    spawn_state: Res<State<SpawnState>>,
    attach_state: Res<State<AttachState>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if spawn_state.get() == &SpawnState::Done && attach_state.get() == &AttachState::Attached {
        app_state.set(AppState::Playing);
    }
}

fn start_new_game(
    mut app_state: ResMut<NextState<AppState>>,
    mut mischief_events: EventReader<MischiefEvent>,
) {
    for event in mischief_events.iter() {
        if let MischiefEventData::Button {
            button: _,
            pressed: true,
        } = event.event_data
        {
            app_state.set(AppState::Playing);
        }
    }
}

fn on_enter_link(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::Init);
}

fn on_exit_link(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::None);
}

#[derive(Component)]
pub struct DespawnOnExitInit;

#[derive(Component)]
pub struct DespawnOnExitGameOver;

#[derive(Component)]
pub struct DespawnOnExitLink;
