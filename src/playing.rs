use bevy::prelude::*;
use rand::Rng;

use crate::{
  window_setup::PlayArea, AppState, Hand, MouseControlConfig, MouseControlled, MOUSE_RADIUS,
  PLAYER_COLOR, RETICLE_COLOR,
};

// MVP tasks:
// Spawn enemies (done)
// Player shoots (done)
// Enemies damage the player
// Player damages enemies (part done, needs collision detection and partial damage)

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(EnemySpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
      )))
      .insert_resource(PlayerShootTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
      )))
      .add_systems(OnEnter(AppState::Playing), init_cursor_roles)
      .add_systems(
        Update,
        (spawn_enemy, shoot).run_if(in_state(AppState::Playing)),
      );
  }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Player;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Reticle;

fn init_cursor_roles(
  mut commands: Commands,
  mut cursors: Query<(Entity, &mut MouseControlled)>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (entity, mut mouse_controlled) in cursors.iter_mut() {
    match mouse_controlled.hand {
      Some(Hand::Left) => {
        commands.entity(entity).insert((
          Player,
          Mesh2d::from(meshes.add(Circle::new(MOUSE_RADIUS))),
          MeshMaterial2d(materials.add(PLAYER_COLOR)),
        ));
        mouse_controlled.physics = MouseControlConfig::WithSpeedLimit(8.);
      }
      Some(Hand::Right) => {
        // TODO make the reticle more reticle-y (e.g. a crosshair or box-corners with a dot in the center)
        // This might require making the mesh from multiple shapes/paths.
        commands.entity(entity).insert((
          Reticle,
          Mesh2d::from(meshes.add(Rectangle::new(2. * MOUSE_RADIUS, 2. * MOUSE_RADIUS))),
          MeshMaterial2d(materials.add(RETICLE_COLOR)),
        ));
        mouse_controlled.physics = MouseControlConfig::Direct;
      }
      None => {}
    }
  }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Enemy;

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

fn spawn_enemy(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  player: Query<&Transform, With<Player>>,
  time: Res<Time>,
  mut timer: ResMut<EnemySpawnTimer>,
  play_area: Res<PlayArea>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let player_position = player.single().translation;
  let min_distance = 2.5;

  let mut rng = rand::thread_rng();
  let enemy_position = std::iter::from_fn(|| {
    Some(rng.gen::<Vec2>() * play_area.size_world - play_area.size_world / 2.0)
  })
  .filter(|&enemy_position| (enemy_position - player_position.truncate()).length() >= min_distance)
  .next()
  .unwrap();

  commands.spawn((
    Enemy,
    Transform::from_translation(enemy_position.extend(0.0)),
    GlobalTransform::default(),
    Mesh2d::from(meshes.add(RegularPolygon::new(0.25, 3))),
    MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
  ));
}

#[derive(Resource)]
struct PlayerShootTimer(Timer);

fn shoot(
  mut commands: Commands,
  player: Query<&Transform, With<Reticle>>,
  enemies: Query<(Entity, &Transform), With<Enemy>>,
  time: Res<Time>,
  mut timer: ResMut<PlayerShootTimer>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let player_position = player.single().translation;

  for (enemy_entity, enemy_transform) in enemies.iter() {
    // TODO actually check for mesh intersection
    if (enemy_transform.translation.xy() - player_position.xy()).length() < 0.5 {
      commands.entity(enemy_entity).despawn();
    }
  }
}
