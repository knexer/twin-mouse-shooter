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
// Figure out which mouse is left and which is right. Maybe ask player to move the mice to the sides of the screen?
// Make the player entity move with the left mouse, and the reticle entity move with the right mouse.
// Make the player entity move with speed/acceleration limits.

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
        )
        .add_systems(
            Update,
            toggle_os_cursor.run_if(input_just_pressed(KeyCode::Backquote)),
        )
        .add_systems(
            Startup,
            (size_window, spawn_camera, toggle_os_cursor).chain(),
        )
        .add_systems(Update, close_on_esc)
        .add_plugins(MischiefPlugin)
        .add_systems(Startup, spawn_cursors)
        .add_systems(
            Update,
            apply_mouse_events
                .after(mischief::poll_events)
                .run_if(input_toggle_active(true, KeyCode::Backquote)),
        )
        .run();
}

#[derive(Component)]
struct MouseControlled(pub Option<u32>);

fn spawn_cursors(
    mut commands: Commands,
    mischief_session: NonSend<MischiefSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (i, mouse) in mischief_session.session.devices.iter().enumerate() {
        commands.spawn((
            Transform::default(),
            MouseControlled(Some(mouse.id)),
            Mesh2d::from(meshes.add(RegularPolygon::new(MOUSE_RADIUS, 3u32 + i as u32))),
            MeshMaterial2d(materials.add(Color::hsl(
                360. * i as f32 / mischief_session.session.devices.len() as f32,
                0.95,
                0.7,
            ))),
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
            if mouse_controlled.0 == Some(event.device) {
                match event.event_data {
                    mischief::MischiefEventData::RelMotion { x, y } => {
                        let valid_positions = Rect::from_corners(
                            window_to_world(window.size()),
                            window_to_world(Vec2::ZERO),
                        )
                        .inflate(-MOUSE_RADIUS);
                        let world_space_delta = window_to_world(Vec2::new(x as f32, y as f32))
                            - window_to_world(Vec2::ZERO);
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
    let next_visible = !window.cursor_options.visible;
    window.set_cursor_position(Some(window_center));
    window.cursor_options = bevy::window::CursorOptions {
        visible: next_visible,
        grab_mode: match next_visible {
            true => bevy::window::CursorGrabMode::None,
            false => bevy::window::CursorGrabMode::Locked,
        },
        ..default()
    };
}

fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(BACKGROUND_COLOR));
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 1.0 / PIXELS_PER_METER,
            ..OrthographicProjection::default_2d()
        },
    ));
}
