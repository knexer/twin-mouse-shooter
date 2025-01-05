use bevy::prelude::*;

use crate::{
  mischief::{MischiefEvent, MischiefEventData},
  AppState,
};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::GameOver), spawn_game_over_text)
      .add_systems(Update, start_new_game);
  }
}

fn spawn_game_over_text(mut commands: Commands) {
  println!("Game Over");
  commands
    .spawn((
      TextLayout {
        justify: JustifyText::Center,

        ..default()
      },
      Text::default(),
      Node {
        position_type: PositionType::Absolute,
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        ..default()
      },
      StateScoped(AppState::GameOver),
    ))
    .with_child((
      TextSpan::new("Game Over!\n"),
      TextFont {
        font_size: 60.0,
        ..default()
      },
    ))
    .with_child((
      TextSpan::new("Click to restart"),
      TextFont {
        font_size: 20.0,
        ..default()
      },
    ));
}

fn start_new_game(
  mut next_state: ResMut<NextState<AppState>>,
  mut click_events: EventReader<MischiefEvent>,
) {
  for MischiefEvent {
    device: _,
    event_data,
  } in click_events.read()
  {
    if let MischiefEventData::Button {
      button: _,
      pressed: true,
    } = event_data
    {
      next_state.set(AppState::Playing);
    };
  }
}
