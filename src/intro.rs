use bevy::prelude::*;

use crate::{
  apply_mouse_events, mischief::MischiefSession, window_to_world, AppState, Hand, MouseControlled,
  MOUSE_RADIUS,
};

// Needs some wrapping up / polish:
// - spawn some boxes for the mouse assignment targets
// - despawn each box when the corresponding cursor is assigned
// - show some intro text / instructions (in each box, and in the middle of the screen)
// - change states when we have both mice assigned
// - make the text and leftover cursors disappear when we change states
// - spawn the cursors with some vertical spread instead of all on top of each other

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Intro), spawn_cursors)
      .add_systems(
        Update,
        (assign_cursor_hands, color_cursors)
          .chain()
          .after(apply_mouse_events)
          .run_if(in_state(AppState::Intro)),
      );
  }
}

fn spawn_cursors(
  mut commands: Commands,
  mischief_session: NonSend<MischiefSession>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (i, mouse) in mischief_session.session.devices.iter().enumerate() {
    commands.spawn((
      Transform::default(),
      MouseControlled {
        id: mouse.id,
        hand: None,
      },
      Mesh2d::from(meshes.add(RegularPolygon::new(MOUSE_RADIUS, 3u32 + i as u32))),
      MeshMaterial2d(materials.add(Color::WHITE)),
    ));
  }
}

fn assign_cursor_hands(
  mut mouse_controlled: Query<(&Transform, &mut MouseControlled)>,
  window_query: Query<&Window>,
  camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
  let window = window_query.single();
  let window_to_world = {
    let (camera_transform, projection) = camera_query.single();
    |position: Vec2| window_to_world(window, camera_transform, projection, position)
  };

  let needs_left = !mouse_controlled
    .iter()
    .any(|(_, mc)| mc.hand == Some(Hand::Left));
  let needs_right = !mouse_controlled
    .iter()
    .any(|(_, mc)| mc.hand == Some(Hand::Right));

  for (transform, mut mouse_controlled) in mouse_controlled.iter_mut() {
    if mouse_controlled.hand.is_some() {
      continue;
    }

    if needs_left
      && transform.translation.x < window_to_world(Vec2::new(window.width() / 4.0, 0.0)).x
    {
      mouse_controlled.hand = Some(Hand::Left);
    } else if needs_right
      && transform.translation.x > window_to_world(Vec2::new(window.width() * 3.0 / 4.0, 0.0)).x
    {
      mouse_controlled.hand = Some(Hand::Right);
    }
  }
}

fn color_cursors(
  mouse_controlled: Query<(&Transform, &MouseControlled, &MeshMaterial2d<ColorMaterial>)>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  window_query: Query<&Window>,
  camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
  let window = window_query.single();
  let window_to_world = {
    let (camera_transform, projection) = camera_query.single();
    |position: Vec2| window_to_world(window, camera_transform, projection, position)
  };

  let needs_left = !mouse_controlled
    .iter()
    .any(|(_, mc, _)| mc.hand == Some(Hand::Left));
  let needs_right = !mouse_controlled
    .iter()
    .any(|(_, mc, _)| mc.hand == Some(Hand::Right));

  let left_color = Color::hsl(30., 0.95, 0.7);
  let left_base_none_color = Color::hsl(30., 0.0, 0.7);
  let right_color = Color::hsl(150., 0.95, 0.7);
  let right_base_none_color = Color::hsl(150., 0.0, 0.7);
  for (transform, mouse_controlled, material) in mouse_controlled.iter() {
    let new_color = match mouse_controlled.hand {
      Some(Hand::Left) => left_color,
      Some(Hand::Right) => right_color,
      None => {
        let left_edge = window_to_world(Vec2::new(window.width() / 4.0, 0.0)).x;
        let right_edge = window_to_world(Vec2::new(window.width() * 3.0 / 4.0, 0.0)).x;
        let center = (left_edge + right_edge) / 2.0;
        let proportion_to_right = (transform.translation.x - center) / (right_edge - center);
        let proportion_to_left = -proportion_to_right;

        if proportion_to_left > 0. && needs_left {
          left_base_none_color.mix(&left_color, proportion_to_left)
        } else if proportion_to_right > 0. && needs_right {
          right_base_none_color.mix(&right_color, proportion_to_right)
        } else {
          left_base_none_color
        }
      }
    };
    materials.insert(material, new_color.into());
  }
}
