use bevy::prelude::*;

use crate::{
  AppState, Hand, MouseControlConfig, MouseControlled, MOUSE_RADIUS, PLAYER_COLOR, RETICLE_COLOR,
};

// MVP tasks:
// Spawn enemies
// Player shoots
// Enemies damage the player
// Player damages enemies

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(EnemySpawnTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
      )))
      .add_systems(OnEnter(AppState::Playing), init_cursor_roles)
      .add_systems(Update, spawn_enemy.run_if(in_state(AppState::Playing)));
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
) {
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  let player_position = player.single().translation;

  // TODO spawn at a random location not too near the player
  let enemy_position = Vec3::new(
    player_position.x + 1.0,
    player_position.y,
    player_position.z,
  );

  commands.spawn((
    Enemy,
    Transform::from_translation(enemy_position),
    GlobalTransform::default(),
    Mesh2d::from(meshes.add(RegularPolygon::new(0.25, 3))),
    MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
  ));
}
