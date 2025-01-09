use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};

use crate::playing::{Enemy, Player};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      (
        (contact_damage, damage_enemies_in_area).in_set(ApplyDamageSet),
        damage_flicker,
      ),
    );
  }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyDamageSet;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct DamageArea {
  pub damage: u32,
  pub half_size: Vec2,
}

fn damage_enemies_in_area(
  mut commands: Commands,
  damage_areas: Query<(Entity, &Transform, &DamageArea)>,
  mut enemies: Query<(&Transform, &mut Enemy)>,
) {
  for (entity, area_transform, area) in damage_areas.iter() {
    let world_to_area = area_transform.compute_matrix().inverse();
    for (enemy_transform, mut enemy) in enemies.iter_mut() {
      let enemy_pos_in_area = world_to_area.transform_point3(enemy_transform.translation);

      if enemy_pos_in_area.x > -area.half_size.x
        && enemy_pos_in_area.x < area.half_size.x
        && enemy_pos_in_area.y > -area.half_size.y
        && enemy_pos_in_area.y < area.half_size.y
      {
        enemy.hp = enemy.hp.saturating_sub(area.damage);
      }
    }
    commands.entity(entity).remove::<DamageArea>();
    let damage_area_shape = ShapeBuilder::new()
      .add(&shapes::Rectangle {
        extents: area.half_size * 2.,
        origin: shapes::RectangleOrigin::Center,
        radii: None,
      })
      .fill(Color::srgba(1.0, 1.0, 1.0, 0.2))
      .build();
    commands.entity(entity).insert((
      DamageFlicker {
        flicker_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        total_duration: Timer::from_seconds(0.2, TimerMode::Once),
      },
      damage_area_shape,
      Visibility::Inherited,
    ));
  }
}

fn contact_damage(
  mut player: Query<(&Transform, &mut Player)>,
  mut enemies: Query<(&Transform, &mut Enemy)>,
) {
  // TODO the damage rate is a fun effect, but it's frame rate dependent
  for (enemy_transform, mut enemy) in enemies.iter_mut() {
    let (player_transform, mut player) = player.single_mut();
    if (enemy_transform.translation.xy() - player_transform.translation.xy()).length() < 0.5 {
      player.hp = player.hp.saturating_sub(1);
      enemy.hp = enemy.hp.saturating_sub(1);
    }
  }
}

#[derive(Component, Debug, Clone, PartialEq)]
struct DamageFlicker {
  flicker_timer: Timer,
  total_duration: Timer,
}

fn damage_flicker(
  mut commands: Commands,
  time: Res<Time>,
  mut query: Query<(Entity, &mut Visibility, &mut DamageFlicker)>,
) {
  for (entity, mut visible, mut flicker) in query.iter_mut() {
    flicker.flicker_timer.tick(time.delta());
    flicker.total_duration.tick(time.delta());

    if flicker.total_duration.finished() {
      commands.entity(entity).despawn_recursive();
    } else if flicker.flicker_timer.finished() {
      *visible = match *visible {
        Visibility::Inherited => Visibility::Hidden,
        Visibility::Hidden => Visibility::Inherited,
        Visibility::Visible => Visibility::Visible,
      };
      flicker.flicker_timer.reset();
    }
  }
}
