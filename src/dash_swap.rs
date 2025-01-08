use bevy::prelude::*;
use bevy_prototype_lyon::{
  prelude::{ShapeBuilder, ShapeBuilderBase},
  shapes,
};

use crate::{
  playing::{DamageArea, MovesStuffSet, Player, Reticle},
  AppState, MouseControlled,
};

pub struct DashSwapPlugin;

impl Plugin for DashSwapPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Playing), init_resources)
      .add_systems(
        Update,
        dash_swap
          .in_set(MovesStuffSet)
          .run_if(in_state(AppState::Playing)),
      );
  }
}

#[derive(Resource)]
struct PlayerDashTimer(Timer);

fn init_resources(mut commands: Commands) {
  commands.insert_resource(PlayerDashTimer(Timer::from_seconds(
    1.0,
    TimerMode::Repeating,
  )));
}

// Swap the player and reticle on a timer, and damage enemies in the player's path.
fn dash_swap(
  mut commands: Commands,
  mut dash_timer: ResMut<PlayerDashTimer>,
  time: Res<Time>,
  mut player: Query<(&mut Transform, &mut MouseControlled), (With<Player>, Without<Reticle>)>,
  mut reticle: Query<(&mut Transform, &mut MouseControlled), (With<Reticle>, Without<Player>)>,
) {
  if !dash_timer.0.tick(time.delta()).just_finished() {
    return;
  }

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

  // Damage enemies in the player's path.
  let player_to_reticle = (reticle_transform.translation - player_transform.translation).truncate();
  let mid_position = player_transform.translation.truncate() + player_to_reticle / 2.0;
  let half_width = player_to_reticle.length() / 2.0 + 0.5;
  let half_height = 0.5;

  commands.spawn((
    Transform::from_translation(mid_position.extend(0.0))
      .with_rotation(Quat::from_rotation_z(player_to_reticle.to_angle())),
    DamageArea {
      damage: 10,
      half_size: Vec2::new(half_width, half_height),
    },
    ShapeBuilder::new()
      .add(&shapes::Rectangle {
        extents: Vec2::new(half_width * 2., half_height * 2.),
        ..default()
      })
      .fill(Color::WHITE)
      .build(),
  ));
}
