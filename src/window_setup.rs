use bevy::{
  input::common_conditions::{input_just_pressed, input_toggle_active},
  prelude::*,
  window::WindowResolution,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
        Startup,
        (size_window, spawn_camera, toggle_os_cursor).chain(),
      )
      .add_systems(Update, close.run_if(input_just_pressed(KeyCode::Escape)))
      .add_plugins(
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
      );
  }
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
