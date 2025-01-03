use bevy::prelude::*;

use crate::{
  apply_mouse_events,
  mischief::MischiefSession,
  path::{Path, WindDirection},
  window_to_world, AppState, Hand, MouseControlled, MOUSE_RADIUS, PLAYER_COLOR, RETICLE_COLOR,
  UNASSIGNED_COLOR,
};

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::Intro), (spawn_cursors, spawn_boxes))
      .add_systems(
        Update,
        (assign_cursor_hands, (color_cursors, progress_intro))
          .chain()
          .after(apply_mouse_events)
          .run_if(in_state(AppState::Intro)),
      )
      .add_systems(OnExit(AppState::Intro), cleanup_intro);
  }
}

fn make_box_mesh(outer_size: Vec2, border_thickness: f32, outer_corner_radius: f32) -> Mesh {
  assert!(outer_corner_radius > border_thickness);

  let left_edge = -outer_size.x / 2.0;
  let right_edge = outer_size.x / 2.0;
  let top_edge = outer_size.y / 2.0;
  let bottom_edge = -outer_size.y / 2.0;

  let left_curve = left_edge + outer_corner_radius;
  let right_curve = right_edge - outer_corner_radius;
  let top_curve = top_edge - outer_corner_radius;
  let bottom_curve = bottom_edge + outer_corner_radius;

  let inside_left_edge = left_edge + border_thickness;
  let inside_right_edge = right_edge - border_thickness;
  let inside_top_edge = top_edge - border_thickness;
  let inside_bottom_edge = bottom_edge + border_thickness;

  let mut path: Path = Path::new();
  // Draw the outer rectangle (with rounded corners)
  // Going counterclockwise from the top left before the bevel
  path.move_to(Vec2::new(left_curve, top_edge));
  path.arc_to(
    Vec2::new(left_edge, top_curve),
    Vec2::new(left_curve, top_curve),
    8,
    WindDirection::CounterClockwise,
  );
  path.line_to(Vec2::new(left_edge, bottom_curve));
  path.arc_to(
    Vec2::new(left_curve, bottom_edge),
    Vec2::new(left_curve, bottom_curve),
    8,
    WindDirection::CounterClockwise,
  );
  path.line_to(Vec2::new(right_curve, bottom_edge));
  path.arc_to(
    Vec2::new(right_edge, bottom_curve),
    Vec2::new(right_curve, bottom_curve),
    8,
    WindDirection::CounterClockwise,
  );
  path.line_to(Vec2::new(right_edge, top_curve));
  path.arc_to(
    Vec2::new(right_curve, top_edge),
    Vec2::new(right_curve, top_curve),
    8,
    WindDirection::CounterClockwise,
  );
  path.line_to(Vec2::new(left_curve + 0.0001, top_edge));

  // Now the inner rectangle (again with rounded corners)
  // And we go clockwise from the top left
  path.line_to(Vec2::new(left_curve + 0.0001, inside_top_edge));
  path.line_to(Vec2::new(right_curve, inside_top_edge));
  path.arc_to(
    Vec2::new(inside_right_edge, top_curve),
    Vec2::new(right_curve, top_curve),
    8,
    WindDirection::Clockwise,
  );
  path.line_to(Vec2::new(inside_right_edge, bottom_curve));
  path.arc_to(
    Vec2::new(right_curve, inside_bottom_edge),
    Vec2::new(right_curve, bottom_curve),
    8,
    WindDirection::Clockwise,
  );
  path.line_to(Vec2::new(left_curve, inside_bottom_edge));
  path.arc_to(
    Vec2::new(inside_left_edge, bottom_curve),
    Vec2::new(left_curve, bottom_curve),
    8,
    WindDirection::Clockwise,
  );
  path.line_to(Vec2::new(inside_left_edge, top_curve));
  path.arc_to(
    Vec2::new(left_curve, inside_top_edge),
    Vec2::new(left_curve, top_curve),
    8,
    WindDirection::Clockwise,
  );
  path.close();

  path.build_triangle_mesh()
}

fn spawn_boxes(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  window: Query<&Window>,
  camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
  let window = window.single();
  let window_to_world = {
    let (camera_transform, projection) = camera_query.single();
    |position: Vec2| window_to_world(window, camera_transform, projection, position)
  };

  let world_size = (window_to_world(Vec2::new(window.width(), window.height()))
    - window_to_world(Vec2::ZERO))
  .abs();

  let inset_world = 0.5;
  let rect_size = Vec2::new(
    world_size.x / 4.0 - inset_world,
    world_size.y - 2. * inset_world,
  );
  let box_handle = meshes.add(make_box_mesh(rect_size, 0.05, 0.5));

  let left_center = window_to_world(Vec2::new(window.width() / 8.0, window.height() / 2.0))
    + Vec2::new(inset_world / 2., 0.0);
  commands.spawn((
    Transform::from_translation(left_center.extend(0.0)),
    Mesh2d::from(box_handle.clone()),
    MeshMaterial2d(materials.add(PLAYER_COLOR)),
    DespawnOnHandAssignment(Hand::Left),
  ));

  let right_center = window_to_world(Vec2::new(window.width() * 7.0 / 8.0, window.height() / 2.0))
    - Vec2::new(inset_world / 2., 0.0);
  commands.spawn((
    Transform::from_translation(right_center.extend(0.0)),
    Mesh2d::from(box_handle),
    MeshMaterial2d(materials.add(RETICLE_COLOR)),
    DespawnOnHandAssignment(Hand::Right),
  ));
}

fn spawn_cursors(
  mut commands: Commands,
  mischief_session: NonSend<MischiefSession>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (i, mouse) in mischief_session.session.devices.iter().enumerate() {
    commands.spawn((
      Transform::from_translation(Vec3::new(
        0.0,
        -4. * i as f32 / mischief_session.session.devices.len() as f32,
        0.0,
      )),
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

  for (transform, mouse_controlled, material) in mouse_controlled.iter() {
    let new_color = match mouse_controlled.hand {
      Some(Hand::Left) => PLAYER_COLOR,
      Some(Hand::Right) => RETICLE_COLOR,
      None => {
        let left_edge = window_to_world(Vec2::new(window.width() / 4.0, 0.0)).x;
        let right_edge = window_to_world(Vec2::new(window.width() * 3.0 / 4.0, 0.0)).x;
        let center = (left_edge + right_edge) / 2.0;
        let proportion_to_right = (transform.translation.x - center) / (right_edge - center);
        let proportion_to_left = -proportion_to_right;

        if proportion_to_left > 0. && needs_left {
          UNASSIGNED_COLOR.mix(&PLAYER_COLOR, proportion_to_left)
        } else if proportion_to_right > 0. && needs_right {
          UNASSIGNED_COLOR.mix(&RETICLE_COLOR, proportion_to_right)
        } else {
          UNASSIGNED_COLOR
        }
      }
    };
    materials.insert(material, new_color.into());
  }
}

#[derive(Component)]
struct DespawnOnHandAssignment(Hand);

fn progress_intro(
  mut commands: Commands,
  boxes: Query<(Entity, &DespawnOnHandAssignment)>,
  hands: Query<&MouseControlled>,
  mut exit_intro: ResMut<NextState<AppState>>,
) {
  let left_assigned = hands.iter().any(|mc| mc.hand == Some(Hand::Left));
  let right_assigned = hands.iter().any(|mc| mc.hand == Some(Hand::Right));
  for (entity, DespawnOnHandAssignment(hand)) in boxes.iter() {
    if (hand == &Hand::Left && left_assigned) || (hand == &Hand::Right && right_assigned) {
      commands.entity(entity).despawn();
    }
  }

  if left_assigned && right_assigned {
    exit_intro.set(AppState::Playing);
  }
}

fn cleanup_intro(mut commands: Commands, cursors: Query<(Entity, &MouseControlled)>) {
  for (id, mc) in cursors.iter() {
    match mc.hand {
      Some(_) => {
        commands
          .entity(id)
          .remove::<Mesh2d>()
          .remove::<MeshMaterial2d<ColorMaterial>>();
      }
      None => {
        commands.entity(id).despawn();
      }
    }
  }
}
