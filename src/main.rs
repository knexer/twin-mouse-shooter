use bevy::{
  input::common_conditions::{input_just_pressed, input_toggle_active},
  prelude::*,
  window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use mischief::{MischiefEvent, MischiefPlugin, MischiefSession};

mod mischief;

const PIXELS_PER_METER: f32 = 100.0;
pub const BACKGROUND_COLOR: Color = Color::srgb(64.0 / 255.0, 67.0 / 255.0, 78.0 / 255.0);
const MOUSE_RADIUS: f32 = 0.4;

// MVP features:
// 2D, top down, fixed camera, real time game.
// Downloadable for Mac and, hopefully, Windows. Can't support web because of the weird input APIs needed.
// There is a player entity. Its movement is zero-order controlled with the mouse (probably with speed/acceleration limits).
// There is a targeting reticle entity. Its movement is zero-order controlled with the other mouse (no speed limit).
// There are hazards the player must avoid.
// There are things the player must shoot (with the reticle).
// Clicking swaps the locations of the player and the reticle (likely with some restrictions).
// Enemies spawn and attack the player. The player can take damage and/or die. It's a bullet hell kind of thing.
// The player shoots at their reticle's location automatically, and can damage and kill the enemies.

// MVP tasks:
// Set up project and import library for multiple mice. (done)
// Control one entity with each discovered mouse, no speed limits. (done)
// Clamp the entities to the screen. (done)
// Figure out which mouse is left and which is right. (done, needs polish)
// Make the player entity move with the left mouse, and the reticle entity move with the right mouse.
// Make the player entity move with speed/acceleration limits.

// Next steps:
// Just finished assigning left/right/none to the mice.
// Needs some wrapping up / polish:
// - move intro state stuff to a new plugin
// - spawn some boxes for the mouse assignment targets
// - despawn each box when the corresponding cursor is assigned
// - show some intro text / instructions (in each box, and in the middle of the screen)
// - change states when we have both mice assigned
// - make the text and leftover cursors disappear when we change states
// - spawn the cursors with some vertical spread instead of all on top of each other

// Maybe, while we're at it, move some of the basic window/camera/cursor stuff to a new plugin.
// It would be nice to reserve main.rs for doing new things.

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .insert_state(AppState::Intro)
    .add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)))
    .add_systems(
      Update,
      toggle_os_cursor.run_if(input_just_pressed(KeyCode::Backquote)),
    )
    .add_systems(
      Startup,
      (size_window, spawn_camera, toggle_os_cursor).chain(),
    )
    .add_systems(Update, close.run_if(input_just_pressed(KeyCode::Escape)))
    .add_plugins(MischiefPlugin)
    .add_systems(OnEnter(AppState::Intro), spawn_cursors)
    .add_systems(
      Update,
      apply_mouse_events
        .after(mischief::poll_events)
        .run_if(input_toggle_active(true, KeyCode::Backquote)),
    )
    .add_systems(
      Update,
      (assign_cursor_hands, color_cursors)
        .chain()
        .after(apply_mouse_events)
        .run_if(in_state(AppState::Intro)),
    )
    .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
  Intro,
  Playing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Hand {
  Left,
  Right,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct MouseControlled {
  pub id: u32,
  pub hand: Option<Hand>,
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

fn apply_mouse_events(
  mut mouse_events: EventReader<MischiefEvent>,
  mut mouse_controlled: Query<(&mut Transform, &MouseControlled)>,
  window_query: Query<&Window>,
  camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
  let window = window_query.single();
  let window_to_world = {
    let (camera_transform, projection) = camera_query.single();
    |position: Vec2| window_to_world(window, camera_transform, projection, position)
  };

  for event in mouse_events.read() {
    for (mut transform, mouse_controlled) in mouse_controlled.iter_mut() {
      if mouse_controlled.id != event.device {
        continue;
      }
      match event.event_data {
        mischief::MischiefEventData::RelMotion { x, y } => {
          let valid_positions =
            Rect::from_corners(window_to_world(window.size()), window_to_world(Vec2::ZERO))
              .inflate(-MOUSE_RADIUS);
          let world_space_delta =
            window_to_world(Vec2::new(x as f32, y as f32)) - window_to_world(Vec2::ZERO);
          let next_pos = (transform.translation.xy() + world_space_delta)
            .clamp(valid_positions.min, valid_positions.max);

          transform.translation = next_pos.extend(0.0);
        }
        mischief::MischiefEventData::Disconnect => {
          panic!("Mouse disconnected!");
        }
        _ => {}
      }
    }
  }
}

fn window_to_world(
  window: &Window,
  camera_transform: &GlobalTransform,
  projection: &OrthographicProjection,
  position: Vec2,
) -> Vec2 {
  let center = camera_transform.translation().truncate();
  let half_width = (window.width() / 2.0) * projection.scale;
  let half_height = (window.height() / 2.0) * projection.scale;
  let left = center.x - half_width;
  let bottom = center.y - half_height;
  Vec2::new(
    left + position.x * projection.scale,
    bottom + (window.height() - position.y) * projection.scale,
  )
}

fn size_window(mut windows: Query<&mut Window>) {
  let mut window = windows.single_mut();
  let scale_factor = window.scale_factor() as f32;
  window.resolution = WindowResolution::new(1600.0 * scale_factor, 900.0 * scale_factor)
    .with_scale_factor_override(scale_factor as f32);
  window.position.center(MonitorSelection::Current);
}

fn toggle_os_cursor(mut windows: Query<&mut Window>) {
  let mut window = windows.single_mut();
  let window_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
  window.set_cursor_position(Some(window_center));

  let next_visible = !window.cursor_options.visible;
  window.cursor_options = bevy::window::CursorOptions {
    visible: next_visible,
    grab_mode: match next_visible {
      true => bevy::window::CursorGrabMode::None,
      false => bevy::window::CursorGrabMode::Locked,
    },
    ..default()
  };
}

fn close(mut commands: Commands, windows: Query<Entity, With<Window>>) {
  for window in windows.iter() {
    commands.entity(window).despawn();
  }
}

fn spawn_camera(mut commands: Commands) {
  commands.insert_resource(ClearColor(BACKGROUND_COLOR));
  commands.spawn((
    Camera2d,
    OrthographicProjection {
      scale: 1.0 / PIXELS_PER_METER,
      ..OrthographicProjection::default_2d()
    },
  ));
}
