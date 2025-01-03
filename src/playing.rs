use bevy::prelude::*;
use rand::Rng;

use crate::{
  window_setup::PlayArea, AppState, Hand, MouseControlConfig, MouseControlled, MOUSE_RADIUS,
  PLAYER_COLOR, RETICLE_COLOR,
};

// MVP tasks:
// Spawn enemies (done)
// Player shoots (done)
// Player damages enemies, enemies die (done)
// Enemies damage the player (done)
// Player dies, game over (done)

// Killed enemies should increase score
// Game over screen, show score, click to restart
// Enemies should move (random direction, with spin, asteroids style)
// Enemies should shoot
// Show player/enemy health

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
        (
          shoot,
          despawn_dead_enemies,
          enemies_damage_player,
          despawn_dead_enemies,
          spawn_enemy,
          game_over,
        )
          .chain()
          .run_if(in_state(AppState::Playing)),
      );
  }
}

#[derive(Component, Debug, Clone, PartialEq)]
struct Player {
  hp: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
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
          Player { hp: 10. },
          Mesh2d::from(meshes.add(Circle::new(MOUSE_RADIUS))),
          MeshMaterial2d(materials.add(PLAYER_COLOR)),
          StateScoped(AppState::Playing),
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
          StateScoped(AppState::Playing),
        ));
        mouse_controlled.physics = MouseControlConfig::Direct;
      }
      None => {}
    }
  }
}

#[derive(Component, Debug, Clone, PartialEq)]
struct Enemy {
  hp: f32,
}

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
    Enemy { hp: 3. },
    Transform::from_translation(enemy_position.extend(0.0)),
    GlobalTransform::default(),
    Mesh2d::from(meshes.add(RegularPolygon::new(0.25, 3))),
    MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
    StateScoped(AppState::Playing),
  ));
}

#[derive(Resource)]
struct PlayerShootTimer(Timer);

fn shoot(
  player: Query<&Transform, With<Reticle>>,
  mut enemies: Query<(&Transform, &mut Enemy)>,
  time: Res<Time>,
  mut timer: ResMut<PlayerShootTimer>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let player_position = player.single().translation;

  for (enemy_transform, mut enemy) in enemies.iter_mut() {
    // TODO maybe check for collision more better?
    if (enemy_transform.translation.xy() - player_position.xy()).length() < 0.5 {
      enemy.hp -= 1.;
    }
  }
}

fn despawn_dead_enemies(mut commands: Commands, enemies: Query<(Entity, &Enemy)>) {
  for (entity, enemy) in enemies.iter() {
    if enemy.hp <= 0. {
      commands.entity(entity).despawn();
    }
  }
}

fn enemies_damage_player(
  mut player: Query<(&Transform, &mut Player)>,
  mut enemies: Query<(&Transform, &mut Enemy)>,
) {
  for (enemy_transform, mut enemy) in enemies.iter_mut() {
    let (player_transform, mut player) = player.single_mut();
    if (enemy_transform.translation.xy() - player_transform.translation.xy()).length() < 0.5 {
      player.hp -= 1.;
      enemy.hp -= 1.;
    }
  }
}

fn game_over(player: Query<&Player>, mut next_state: ResMut<NextState<AppState>>) {
  let player = player.single();

  if player.hp <= 0. {
    next_state.set(AppState::Intro);
  }
}
