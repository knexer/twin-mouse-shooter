use bevy::prelude::*;

use crate::{
  mischief::{MischiefEvent, MischiefEventData},
  playing::{ApplyDamageSet, DamageArea, MovesStuffSet, Player, Reticle},
  AppState, EnableStateScopedResource, MouseControlled,
};

pub struct ShootPlugin;

impl Plugin for ShootPlugin {
  fn build(&self, app: &mut App) {
    app
      .enable_state_scoped_resource::<PlayerShootTimer>(AppState::Playing)
      .add_systems(OnEnter(AppState::Playing), init_resources)
      .add_systems(
        Update,
        (
          shoot.after(MovesStuffSet).before(ApplyDamageSet),
          swap.in_set(MovesStuffSet),
        )
          .run_if(in_state(AppState::Playing)),
      );
  }
}

fn init_resources(mut commands: Commands) {
  commands.insert_resource(PlayerShootTimer(Timer::from_seconds(
    0.05,
    TimerMode::Repeating,
  )));
}

#[derive(Resource)]
struct PlayerShootTimer(Timer);

fn shoot(
  mut commands: Commands,
  player: Query<&Transform, With<Reticle>>,
  time: Res<Time>,
  mut timer: ResMut<PlayerShootTimer>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let player_position = player.single().translation;

  commands.spawn((
    Transform::from_translation(player_position),
    DamageArea {
      damage: 1,
      half_size: Vec2::new(0.5, 0.5),
    },
  ));
}

fn swap(
  mut player: Query<(&mut Transform, &mut MouseControlled), (With<Player>, Without<Reticle>)>,
  mut reticle: Query<(&mut Transform, &mut MouseControlled), (With<Reticle>, Without<Player>)>,
  mut mouse_events: EventReader<MischiefEvent>,
) {
  for MischiefEvent {
    device: _,
    event_data,
  } in mouse_events.read()
  {
    let MischiefEventData::Button {
      button: _,
      pressed: true,
    } = event_data
    else {
      continue;
    };

    let (mut player_transform, mut player_control) = player.single_mut();
    let (mut reticle_transform, mut reticle_control) = reticle.single_mut();

    std::mem::swap(
      &mut player_transform.translation,
      &mut reticle_transform.translation,
    );
    std::mem::swap(
      &mut player_transform.rotation,
      &mut reticle_transform.rotation,
    );

    std::mem::swap(&mut player_control.id, &mut reticle_control.id);
    std::mem::swap(&mut player_control.hand, &mut reticle_control.hand);
  }
}
