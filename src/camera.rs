use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::{Arena, Z_DARK};
use crate::fx::ScreenShake;
use crate::player::Player;
use crate::states::{GameSet, gameplay_active};

/// At 1080p this shows a 640x360 world view, scaling each art pixel 3x.
const VIEW_SCALE: f32 = 1.0 / 3.0;
const MAX_VIEW_SCALE: f32 = VIEW_SCALE * 3.0;
const ZOOM_SENSITIVITY: f32 = 0.12;

/// Z of the camera entity itself. The night overlay is a child, so its
/// local Z is `Z_DARK - CAMERA_Z` to land on the world `Z_DARK` layer.
const CAMERA_Z: f32 = 900.0;

/// How far the camera leans toward the mouse, in world units.
const LOOK_AHEAD: f32 = 46.0;

const DARK_COLOR: Color = Color::srgba(0.02, 0.03, 0.07, 0.62);

#[derive(Component)]
pub struct CameraRig {
    pub base: Vec2,
}

#[derive(Component)]
pub struct OffscreenRenderCamera;

/// Full-viewport night tint that follows the camera.
#[derive(Component)]
pub struct NightOverlay;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (camera_zoom, camera_follow, sync_overlay, camera_shake)
                .chain()
                .in_set(GameSet::Effects)
                .run_if(gameplay_active),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = commands.spawn((
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scale: VIEW_SCALE,
            ..OrthographicProjection::default_2d()
        }),
        CameraRig { base: Vec2::ZERO },
        Transform::from_xyz(0.0, 0.0, CAMERA_Z),
    ));

    camera.with_children(|parent| {
        parent.spawn((
            Sprite {
                color: DARK_COLOR,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, Z_DARK - CAMERA_Z),
            NightOverlay,
        ));
    });
}

/// Convert the logical window size to the camera's visible world size.
fn viewport_size(window: &Window, view_scale: f32) -> Vec2 {
    Vec2::new(window.width(), window.height()) * view_scale
}

fn projection_scale(projection: &Projection) -> f32 {
    match projection {
        Projection::Orthographic(projection) => projection.scale,
        _ => VIEW_SCALE,
    }
}

fn adjusted_view_scale(current: f32, scroll_delta: f32) -> f32 {
    (current * (-scroll_delta * ZOOM_SENSITIVITY).exp()).clamp(VIEW_SCALE, MAX_VIEW_SCALE)
}

fn camera_zoom(
    mut scroll: MessageReader<MouseWheel>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut projections: Query<&mut Projection, (With<Camera2d>, Without<OffscreenRenderCamera>)>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll.read() {
        if event.window != *window {
            continue;
        }
        scroll_delta += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        };
    }
    if scroll_delta == 0.0 {
        return;
    }

    let Ok(mut projection) = projections.single_mut() else {
        return;
    };
    if let Projection::Orthographic(projection) = &mut *projection {
        projection.scale = adjusted_view_scale(projection.scale, scroll_delta);
    }
}

fn sync_overlay(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&Projection, (With<Camera2d>, Without<OffscreenRenderCamera>)>,
    mut overlay: Query<(&mut Sprite, &mut Transform), With<NightOverlay>>,
) {
    let size = viewport_size(&window, projection_scale(&camera));
    for (mut sprite, mut transform) in &mut overlay {
        if sprite.custom_size != Some(size) {
            sprite.custom_size = Some(size);
        }
        // local Z so the child lands on the world Z_DARK layer
        transform.translation.z = Z_DARK - CAMERA_Z;
    }
}

fn camera_follow(
    time: Res<Time>,
    arena: Res<Arena>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<
        (&Camera, &GlobalTransform, &Projection),
        (Without<Player>, Without<OffscreenRenderCamera>),
    >,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut rig: Query<&mut CameraRig, (With<Camera2d>, Without<OffscreenRenderCamera>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_rig) = rig.single_mut() else {
        return;
    };

    // lean toward the mouse so the crosshair side stays visible
    let mut focus = player_transform.translation.xy();
    let (cam, cam_transform, projection) = *camera;
    if let Some(cursor) = window.cursor_position() {
        if let Ok(point) = cam.viewport_to_world_2d(cam_transform, cursor) {
            let toward = point - focus;
            let reach = toward.length();
            if reach > 1.0 {
                focus += toward / reach * LOOK_AHEAD.min(reach);
            }
        }
    }

    let half_view = viewport_size(&window, projection_scale(projection)) * 0.5;
    let limit = (arena.half - half_view).max(Vec2::ZERO);
    let target = focus.clamp(-limit, limit);
    let smoothing = 1.0 - (-9.0 * time.delta_secs()).exp();
    camera_rig.base = camera_rig.base.lerp(target, smoothing);
}

fn camera_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    rig: Query<&CameraRig, (With<Camera2d>, Without<OffscreenRenderCamera>)>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<OffscreenRenderCamera>)>,
) {
    let Ok(camera_rig) = rig.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let strength = shake.trauma * shake.trauma * 14.0;
    shake.trauma = (shake.trauma - time.delta_secs() * 2.2).max(0.0);

    let offset = Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0) * strength;

    camera_transform.translation.x = camera_rig.base.x + offset.x;
    camera_transform.translation.y = camera_rig.base.y + offset.y;
}

#[cfg(test)]
mod tests {
    use super::{MAX_VIEW_SCALE, VIEW_SCALE, adjusted_view_scale};

    #[test]
    fn wheel_adjusts_zoom_within_limits() {
        assert!(adjusted_view_scale(VIEW_SCALE, -1.0) > VIEW_SCALE);
        assert!(adjusted_view_scale(MAX_VIEW_SCALE, 1.0) < MAX_VIEW_SCALE);
        assert_eq!(adjusted_view_scale(MAX_VIEW_SCALE, -1.0), MAX_VIEW_SCALE);
        assert_eq!(adjusted_view_scale(VIEW_SCALE, 1.0), VIEW_SCALE);
    }
}
