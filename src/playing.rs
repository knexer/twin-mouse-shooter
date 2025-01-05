use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
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

// Show player/enemy health (fill in the sprite in proportion to health)
// Killed enemies should increase score
// Game over screen, show score, click to restart
// Enemies should shoot maybe?
// Scale up the enemy frequency over time
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
        0.05,
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
          display_player_health,
          despawn_dead_enemies,
          spawn_enemy,
          display_enemy_health,
          game_over,
        )
          .chain()
          .run_if(in_state(AppState::Playing)),
      );
  }
}

const PLAYER_MAX_HP: i32 = 30;

#[derive(Component, Debug, Clone, PartialEq)]
struct Player {
  hp: i32,
}

#[derive(Component, Debug, Clone, PartialEq)]
struct Reticle;

#[derive(Component, Clone)]
struct HealthDisplay {
  shapes: Vec<Shape>,
}

fn init_cursor_roles(mut commands: Commands, mut cursors: Query<(Entity, &mut MouseControlled)>) {
  for (entity, mut mouse_controlled) in cursors.iter_mut() {
    match mouse_controlled.hand {
      Some(Hand::Left) => {
        let shape_for_hp = |hp: i32| -> Shape {
          // Create a circle, filled from the bottom up in proportion to the player's health
          // The top portion is hollow.
          let normalized_hp = hp as f32 / PLAYER_MAX_HP as f32;
          let arc_angle = (normalized_hp * 2. - 1.).asin();
          let bottom_filled = ShapePath::new()
            .move_to(Vec2::new(arc_angle.cos(), arc_angle.sin()) * MOUSE_RADIUS)
            .arc(
              Vec2::ZERO,
              Vec2::splat(MOUSE_RADIUS),
              (std::f32::consts::PI - 2. * arc_angle) - 2. * std::f32::consts::PI, // sweep angle
              0.,
            )
            .close();
          ShapeBuilder::new()
            .add(&bottom_filled)
            .fill(PLAYER_COLOR)
            .build()
        };
        commands
          .entity(entity)
          .insert((
            Player { hp: PLAYER_MAX_HP },
            ShapeBuilder::new()
              .add(&shapes::Circle {
                radius: MOUSE_RADIUS,
                center: Vec2::ZERO,
              })
              .stroke(Stroke {
                color: PLAYER_COLOR,
                options: StrokeOptions::default()
                  .with_line_width(0.05)
                  .with_tolerance(0.02),
              })
              .build(),
            StateScoped(AppState::Playing),
          ))
          .with_child((
            HealthDisplay {
              shapes: (0..=PLAYER_MAX_HP).map(shape_for_hp).collect(),
            },
            shape_for_hp(PLAYER_MAX_HP),
          ));
        mouse_controlled.physics = MouseControlConfig::WithSpeedLimit(8.);
      }
      Some(Hand::Right) => {
        let corner = |x: f32, y: f32| -> ShapePath {
          ShapePath::new()
            .move_to(Vec2::new(x * MOUSE_RADIUS, y * MOUSE_RADIUS / 2.))
            .line_to(Vec2::new(x * MOUSE_RADIUS, y * MOUSE_RADIUS))
            .line_to(Vec2::new(x * MOUSE_RADIUS / 2., y * MOUSE_RADIUS))
        };
        let shape = ShapeBuilder::new()
          .add(&corner(1., 1.))
          .add(&corner(-1., 1.))
          .add(&corner(-1., -1.))
          .add(&corner(1., -1.))
          .add(
            &ShapePath::new()
              .move_to(Vec2::new(0., -0.05))
              .line_to(Vec2::new(0., 0.05)),
          )
          .stroke(Stroke::new(RETICLE_COLOR, 0.1))
          .build();

        commands
          .entity(entity)
          .insert((Reticle, shape, StateScoped(AppState::Playing)));
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
  let max_hp = 10;

  let shape_for_hp = |hp: i32| -> Shape {
    // The enemy is a triangle, filled from the bottom up in proportion to the enemy's health
    // So, the bottom is a trapezoid.
    let normalized_hp = hp as f32 / max_hp as f32;
    let angle = 2. / 3. * std::f32::consts::PI;
    let bottom_left = Vec2::from_angle(angle + PI / 2.).clamp_length_max(0.25);
    let bottom_right = Vec2::from_angle(2. * angle + PI / 2.).clamp_length_max(0.25);
    let top = Vec2::new(0., 1.).clamp_length_max(0.25);
    let bottom_filled = ShapePath::new()
      .move_to(bottom_left)
      .line_to(bottom_left + (top - bottom_left) * normalized_hp)
      .line_to(bottom_right + (top - bottom_right) * normalized_hp)
      .line_to(bottom_right)
      .close();
    ShapeBuilder::new()
      .add(&bottom_filled)
      .fill(Color::hsl(0., 0.95, 0.7))
      .build()
  };

  commands
    .spawn((
      Enemy {
        hp: max_hp,
        velocity: -spawn_direction * rng.gen_range(min_speed..max_speed),
        radial_velocity: rng.gen_range(-max_radial_velocity..max_radial_velocity),
      },
      Transform::from_translation(spawn_position.extend(0.0)),
      GlobalTransform::default(),
      ShapeBuilder::new()
        .add(&shapes::RegularPolygon {
          sides: 3,
          center: Vec2::ZERO,
          feature: RegularPolygonFeature::Radius(0.25),
        })
        .stroke(Stroke::new(Color::hsl(0., 0.95, 0.7), 0.05))
        .build(),
      StateScoped(AppState::Playing),
    ))
    .with_child((
      HealthDisplay {
        shapes: (0..=max_hp).map(shape_for_hp).collect(),
      },
      shape_for_hp(max_hp),
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
      commands.entity(entity).despawn_recursive();
    }
  }
}

fn enemies_damage_player(
  mut player: Query<(&Transform, &mut Player)>,
  mut enemies: Query<(&Transform, &mut Enemy)>,
) {
  // TODO the damage rate is a fun effect, but it's frame rate dependent
  for (enemy_transform, mut enemy) in enemies.iter_mut() {
    let (player_transform, mut player) = player.single_mut();
    if (enemy_transform.translation.xy() - player_transform.translation.xy()).length() < 0.5 {
      player.hp -= 1;
      enemy.hp -= 1;
    }
  }
}

fn display_player_health(
  mut commands: Commands,
  player: Query<(&Player, &Children)>,
  health_displays: Query<(Entity, &HealthDisplay)>,
) {
  let (player, children) = player.single();
  for child in children.iter() {
    let Ok((entity, health_display)) = health_displays.get(*child) else {
      continue;
    };

    commands
      .entity(entity)
      .insert(health_display.shapes[player.hp as usize].clone());
  }
}

fn display_enemy_health(
  mut commands: Commands,
  enemies: Query<(&Enemy, &Children)>,
  health_displays: Query<(Entity, &HealthDisplay)>,
) {
  for (enemy, children) in enemies.iter() {
    for child in children.iter() {
      let Ok((entity, health_display)) = health_displays.get(*child) else {
        continue;
      };

      commands
        .entity(entity)
        .insert(health_display.shapes[enemy.hp as usize].clone());
    }
  }
}

fn game_over(player: Query<&Player>, mut next_state: ResMut<NextState<AppState>>) {
  let player = player.single();

  if player.hp <= 0 {
    next_state.set(AppState::Intro);
  }
}
