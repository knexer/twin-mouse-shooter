use std::collections::HashMap;

use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_prototype_lyon::plugin::ShapePlugin;
use bomb_surprise::BombSurprisePlugin;
use damage::DamagePlugin;
use dash_swap::DashSwapPlugin;
use game_over::GameOverPlugin;
use intro::IntroPlugin;
use mischief::{MischiefEvent, MischiefPlugin};
use playing::{MovesStuffSet, PlayingPlugin};
use shoot::ShootPlugin;
use window_setup::{PlayArea, WindowSetupPlugin};

mod bomb_surprise;
mod damage;
mod dash_swap;
mod game_over;
mod intro;
mod mischief;
mod path;
mod playing;
mod shoot;
mod window_setup;

const MOUSE_RADIUS: f32 = 0.4;
const PLAYER_COLOR: Color = Color::hsl(180., 0.95, 0.7);
const UNASSIGNED_COLOR: Color = Color::hsl(240., 0.95, 0.7);
const RETICLE_COLOR: Color = Color::hsl(300., 0.95, 0.7);

// MVP features:
// 2D, top down, fixed camera, real time game.
// Downloadable for Mac and, hopefully, Windows. Can't support web because of the weird input APIs needed.
// There is a player entity. Its movement is zero-order controlled with the mouse (probably with speed/acceleration limits).
// There is a targeting reticle entity. Its movement is zero-order controlled with the other mouse (no speed limit).
// There are hazards the player must avoid.
// There are things the player must shoot (with the reticle).
// Clicking swaps the locations of the player and the reticle (likely with some restrictions).
// Enemies spawn and attack the player. The player can take damage and/or die. It's a bullet hell kind of thing.
// The player shoots at their reticle's location automatically, and can damage and kill the enemies.

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(ShapePlugin)
    .add_plugins(WindowSetupPlugin)
    .add_plugins(MischiefPlugin)
    .add_plugins(IntroPlugin)
    .add_plugins(PlayingPlugin)
    .add_plugins(DamagePlugin)
    // .add_plugins(ShootPlugin)
    .add_plugins(DashSwapPlugin)
    // .add_plugins(BombSurprisePlugin)
    .add_plugins(GameOverPlugin)
    .insert_state(AppState::Loading)
    .enable_state_scoped_entities::<AppState>()
    .add_event::<CursorMoveEvent>()
    .add_systems(
      Update,
      (
        aggregate_mouse_events,
        apply_mouse_events.in_set(MovesStuffSet),
      )
        .chain()
        .after(mischief::poll_events)
        .run_if(input_toggle_active(true, KeyCode::Backquote)),
    )
    .run();
}

pub trait EnableStateScopedResource {
  fn enable_state_scoped_resource<R: Resource>(&mut self, state: impl States) -> &mut Self;
}

impl EnableStateScopedResource for App {
  fn enable_state_scoped_resource<R: Resource>(&mut self, state: impl States) -> &mut Self {
    self.add_systems(OnExit(state), |world: &mut World| {
      world.remove_resource::<R>();
    })
  }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
  Loading,
  Intro,
  Playing,
  GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Hand {
  Left,
  Right,
}

#[derive(Component, Debug, Clone, PartialEq)]
struct MouseControlled {
  pub id: u32,
  pub hand: Option<Hand>,
  pub physics: MouseControlConfig,
}

#[derive(Debug, Clone, PartialEq)]
enum MouseControlConfig {
  Direct,
  WithSpeedLimit(f32),
}

#[derive(Event, Debug)]
pub struct CursorMoveEvent {
  pub device: u32,
  pub delta_world: Vec2,
}

fn aggregate_mouse_events(
  mut mouse_events: EventReader<MischiefEvent>,
  mut out_events: EventWriter<CursorMoveEvent>,
  play_area: Res<PlayArea>,
) {
  mouse_events
    .read()
    .map(|event| {
      let world_delta = match event.event_data {
        mischief::MischiefEventData::RelMotion { x, y } => {
          (play_area.window_to_world)(Vec2::new(x as f32, y as f32))
            - (play_area.window_to_world)(Vec2::ZERO)
        }
        mischief::MischiefEventData::Disconnect => {
          panic!("Mouse disconnected!");
        }
        _ => Vec2::ZERO,
      };
      (event.device, world_delta)
    })
    .fold(HashMap::new(), |mut map, (id, world_delta)| {
      map
        .entry(id)
        .and_modify(|delta| *delta += world_delta)
        .or_insert(world_delta);
      map
    })
    .iter()
    .for_each(|(device, delta)| {
      out_events.send(CursorMoveEvent {
        device: *device,
        delta_world: *delta,
      });
    });
}

fn apply_mouse_events(
  mut mouse_events: EventReader<CursorMoveEvent>,
  mut mouse_controlled: Query<(&mut Transform, &MouseControlled)>,
  time: Res<Time>,
  play_area: Res<PlayArea>,
) {
  for CursorMoveEvent {
    device,
    delta_world,
  } in mouse_events.read()
  {
    for (mut transform, mc) in mouse_controlled
      .iter_mut()
      .filter(|(_, mc)| mc.id == *device)
    {
      let valid_positions =
        Rect::from_corners(play_area.size_world / 2., play_area.size_world / -2.)
          .inflate(-MOUSE_RADIUS);

      let velocity_clamped_delta_world = match mc.physics {
        MouseControlConfig::Direct => *delta_world,
        MouseControlConfig::WithSpeedLimit(limit) => {
          delta_world.clamp_length(0., limit * time.delta_secs())
        }
      };

      let next_pos = (transform.translation.xy() + velocity_clamped_delta_world)
        .clamp(valid_positions.min, valid_positions.max);

      transform.translation = next_pos.extend(transform.translation.z);
    }
  }
}
