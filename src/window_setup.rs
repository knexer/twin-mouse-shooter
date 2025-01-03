use bevy::{
  input::common_conditions::{input_just_pressed, input_toggle_active},
  prelude::*,
  window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::AppState;

pub struct WindowSetupPlugin;

const PIXELS_PER_METER: f32 = 100.0;
const BACKGROUND_COLOR: Color = Color::srgb(64.0 / 255.0, 67.0 / 255.0, 78.0 / 255.0);

impl Plugin for WindowSetupPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        Update,
        toggle_os_cursor.run_if(input_just_pressed(KeyCode::Backquote)),
      )
      .add_systems(
        OnEnter(AppState::Loading),
        (
          (size_window, spawn_camera),
          (configure_play_area, toggle_os_cursor),
          exit_loading,
        )
          .chain(),
      )
      .add_systems(Update, close.run_if(input_just_pressed(KeyCode::Escape)))
      .add_plugins(
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
      );
  }
}

#[derive(Resource)]
pub struct PlayArea {
  pub size_world: Vec2,
  pub window_to_world: Box<dyn Fn(Vec2) -> Vec2 + Send + Sync>,
}

fn size_window(mut windows: Query<&mut Window>) {
  let mut window = windows.single_mut();
  let scale_factor = window.scale_factor() as f32;
  window.resolution = WindowResolution::new(1600.0 * scale_factor, 900.0 * scale_factor)
    .with_scale_factor_override(scale_factor as f32);
  window.position.center(MonitorSelection::Current);
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

fn configure_play_area(
  mut commands: Commands,
  window_query: Query<&Window>,
  camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
  let window_to_world = {
    let (camera_transform, projection) = camera_query.single();
    let window_size = window_query.single().size();
    let center = camera_transform.translation().truncate();
    let scale = projection.scale;

    let half_width = (window_size.x / 2.0) * scale;
    let half_height = (window_size.y / 2.0) * scale;
    let left = center.x - half_width;
    let bottom = center.y - half_height;
    Box::new(move |position: Vec2| {
      Vec2::new(
        left + position.x * scale,
        bottom + (window_size.y - position.y) * scale,
      )
    })
  };

  commands.insert_resource(PlayArea {
    size_world: (window_to_world(Vec2::new(
      window_query.single().width(),
      window_query.single().height(),
    )) - window_to_world(Vec2::ZERO))
    .abs(),
    window_to_world,
  });
}

fn exit_loading(mut state: ResMut<NextState<AppState>>) {
  state.set(AppState::Intro);
}

fn close(mut commands: Commands, windows: Query<Entity, With<Window>>) {
  for window in windows.iter() {
    commands.entity(window).despawn();
  }
}
