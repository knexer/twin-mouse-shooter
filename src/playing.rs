use bevy::prelude::*;

use crate::{AppState, Hand, MouseControlled, MOUSE_RADIUS, PLAYER_COLOR, RETICLE_COLOR};

// MVP tasks:
// Give player/reticle roles to the cursors. (done)
// Make the player move with speed/acceleration limits.

pub struct PlayingPlugin;

impl Plugin for PlayingPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), init_cursor_roles);
  }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Player;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Reticle;

fn init_cursor_roles(
  mut commands: Commands,
  cursors: Query<(Entity, &MouseControlled)>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (entity, mouse_controlled) in cursors.iter() {
    match mouse_controlled.hand {
      Some(Hand::Left) => {
        commands.entity(entity).with_child((
          Player,
          Mesh2d::from(meshes.add(Circle::new(MOUSE_RADIUS))),
          MeshMaterial2d(materials.add(PLAYER_COLOR)),
        ));
      }
      Some(Hand::Right) => {
        // TODO make the reticle more reticle-y (e.g. a crosshair or box-corners with a dot in the center)
        // This might require making the mesh from multiple shapes/paths.
        commands.entity(entity).with_child((
          Reticle,
          Mesh2d::from(meshes.add(Rectangle::new(2. * MOUSE_RADIUS, 2. * MOUSE_RADIUS))),
          MeshMaterial2d(materials.add(RETICLE_COLOR)),
        ));
      }
      None => {}
    }
  }
}
