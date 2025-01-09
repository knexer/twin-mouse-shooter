use bevy::prelude::*;

use crate::{
  damage::{ApplyDamageSet, DamageArea},
  playing::{MovesStuffSet, Player, Reticle},
  AppState, MouseControlled,
};

pub struct BombSurprisePlugin;

impl Plugin for BombSurprisePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Playing), init_resources)
      .add_systems(
        Update,
        (bomb_swap.in_set(MovesStuffSet), boom.before(ApplyDamageSet))
          .run_if(in_state(AppState::Playing)),
      );
  }
}

#[derive(Resource)]
struct SwapTimer(Timer);

fn init_resources(mut commands: Commands) {
  commands.insert_resource(SwapTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
}

#[derive(Component, Debug)]
struct Bomb {
  damage: u32,
  half_size: Vec2,
  delay: Timer,
}

fn bomb_swap(
  mut commands: Commands,
  mut timer: ResMut<SwapTimer>,
  time: Res<Time>,
  mut player: Query<(&mut Transform, &mut MouseControlled), (With<Player>, Without<Reticle>)>,
  mut reticle: Query<(&mut Transform, &mut MouseControlled), (With<Reticle>, Without<Player>)>,
) {
  // On a timer, swap the player and reticle, and spawn a bomb at the player's previous position.
  // The bomb explodes after a brief delay of its own, damaging enemies in the area.

  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  // Spawn bomb at player's current position
  commands.spawn((
    Transform::from_translation(player.single().0.translation),
    Bomb {
      damage: 10,
      half_size: Vec2::new(1.5, 1.5),
      delay: Timer::from_seconds(0.2, TimerMode::Once),
    },
  ));

  // Swap player and reticle
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

fn boom(
  mut commands: Commands,
  mut bombs: Query<(Entity, &Transform, &mut Bomb)>,
  time: Res<Time>,
) {
  // On a bomb's delay timer, spawn a damage area.
  for (entity, transform, mut bomb) in bombs.iter_mut() {
    if !bomb.delay.tick(time.delta()).just_finished() {
      continue;
    }

    commands.spawn((
      Transform::from_translation(transform.translation),
      DamageArea {
        damage: bomb.damage,
        half_size: bomb.half_size,
      },
    ));

    commands.entity(entity).despawn_recursive();
  }
}
