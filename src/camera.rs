use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::{Arena, Z_DARK};
use crate::fx::ScreenShake;
use crate::player::Player;
use crate::states::{GameSet, gameplay_active};

/// World units visible per window pixel at scale 1.0; 0.5 doubles every
/// sprite texel onto a 2x2 screen pixel block for crisp integer upscaling.
const VIEW_SCALE: f32 = 0.5;

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

/// Full-viewport night tint that follows the camera.
#[derive(Component)]
pub struct NightOverlay;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (camera_follow, sync_overlay, camera_shake)
                .chain()
                .in_set(GameSet::Effects)
                .run_if(gameplay_active),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = commands.spawn((
        Camera2d,
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

/// The overlay lives in camera-local space, so it must cover the viewport
/// in world units: window size (logical px) multiplied by the view scale.
fn viewport_size(window: &Window) -> Vec2 {
    Vec2::new(window.width(), window.height()) * VIEW_SCALE
}

fn sync_overlay(
    window: Single<&Window, With<PrimaryWindow>>,
    mut overlay: Query<(&mut Sprite, &mut Transform), With<NightOverlay>>,
) {
    let size = viewport_size(&window);
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
    camera: Single<(&Camera, &GlobalTransform), Without<Player>>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut rig: Query<&mut CameraRig, With<Camera2d>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_rig) = rig.single_mut() else {
        return;
    };

    // lean toward the mouse so the crosshair side stays visible
    let mut focus = player_transform.translation.xy();
    if let Some(cursor) = window.cursor_position() {
        let (cam, cam_transform) = *camera;
        if let Ok(point) = cam.viewport_to_world_2d(cam_transform, cursor) {
            let toward = point - focus;
            let reach = toward.length();
            if reach > 1.0 {
                focus += toward / reach * LOOK_AHEAD.min(reach);
            }
        }
    }

    let half_view = viewport_size(&window) * 0.5;
    let limit = (arena.half - half_view).max(Vec2::ZERO);
    let target = focus.clamp(-limit, limit);
    let smoothing = 1.0 - (-9.0 * time.delta_secs()).exp();
    camera_rig.base = camera_rig.base.lerp(target, smoothing);
}

fn camera_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    rig: Query<&CameraRig, With<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
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
