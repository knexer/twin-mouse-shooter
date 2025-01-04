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
// Enemies should move (random direction, with spin, asteroids style) (done)

// Killed enemies should increase score
// Game over screen, show score, click to restart
// Enemies should shoot maybe?
// Show player/enemy health
// Click to swap

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
          move_enemies,
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
  hp: i32,
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
          Player { hp: 10 },
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
  hp: i32,
  velocity: Vec2,
  radial_velocity: f32,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

fn spawn_enemy(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  time: Res<Time>,
  mut timer: ResMut<EnemySpawnTimer>,
  play_area: Res<PlayArea>,
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let mut rng = rand::thread_rng();

  // Enemies spawn just outside and move towards the center
  let goal_position = rng.gen::<Vec2>() * play_area.size_world / 2.0 - play_area.size_world / 4.0;
  let spawn_direction = Vec2::from_angle(rng.gen_range(0.0..std::f32::consts::PI * 2.0));
  let spawn_position = {
    let spawn_half_size = play_area.size_world / 2. + Vec2::new(0.5, 0.5);
    let t_x = if spawn_direction.x > 0. {
      (spawn_half_size.x - goal_position.x) / spawn_direction.x
    } else {
      (-spawn_half_size.x - goal_position.x) / spawn_direction.x
    };
    let t_y = if spawn_direction.y > 0. {
      (spawn_half_size.y - goal_position.y) / spawn_direction.y
    } else {
      (-spawn_half_size.y - goal_position.y) / spawn_direction.y
    };
    goal_position + spawn_direction * t_x.min(t_y)
  };

  let min_speed = 1.0;
  let max_speed = 4.0;
  let max_radial_velocity = 3.0;

  commands.spawn((
    Enemy {
      hp: 3,
      velocity: -spawn_direction * rng.gen_range(min_speed..max_speed),
      radial_velocity: rng.gen_range(-max_radial_velocity..max_radial_velocity),
    },
    Transform::from_translation(spawn_position.extend(0.0)),
    GlobalTransform::default(),
    Mesh2d::from(meshes.add(RegularPolygon::new(0.25, 3))),
    MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
    StateScoped(AppState::Playing),
  ));
}

fn move_enemies(mut enemies: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
  for (mut transform, enemy) in enemies.iter_mut() {
    transform.translation += enemy.velocity.extend(0.0) * time.delta_secs();

    let rotation = Quat::from_rotation_z(enemy.radial_velocity * time.delta_secs());
    transform.rotation *= rotation;
  }
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
      enemy.hp -= 1;
    }
  }
}

fn despawn_dead_enemies(mut commands: Commands, enemies: Query<(Entity, &Enemy)>) {
  for (entity, enemy) in enemies.iter() {
    if enemy.hp <= 0 {
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
      player.hp -= 1;
      enemy.hp -= 1;
    }
  }
}

fn game_over(player: Query<&Player>, mut next_state: ResMut<NextState<AppState>>) {
  let player = player.single();

  if player.hp <= 0 {
    next_state.set(AppState::Intro);
  }
}
