use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::Arena;
use crate::fx::ScreenShake;
use crate::player::Player;
use crate::states::{gameplay_active, GameSet};

#[derive(Component)]
pub struct CameraRig {
    pub base: Vec2,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (camera_follow, camera_shake)
                .chain()
                .in_set(GameSet::Effects)
                .run_if(gameplay_active),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        CameraRig {
            base: Vec2::ZERO,
        },
        Transform::from_xyz(0.0, 0.0, 900.0),
    ));
}

fn camera_follow(
    time: Res<Time>,
    arena: Res<Arena>,
    window: Single<&Window, With<PrimaryWindow>>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut rig: Query<&mut CameraRig, With<Camera2d>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_rig) = rig.single_mut() else {
        return;
    };

    let half_view = Vec2::new(window.width(), window.height()) * 0.5;
    let limit = (arena.half - half_view).max(Vec2::ZERO);
    let target = player_transform.translation.xy().clamp(-limit, limit);
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

    let strength = shake.trauma * shake.trauma * 22.0;
    shake.trauma = (shake.trauma - time.delta_secs() * 2.2).max(0.0);

    let offset = Vec2::new(
        fastrand::f32() * 2.0 - 1.0,
        fastrand::f32() * 2.0 - 1.0,
    ) * strength;

    camera_transform.translation.x = camera_rig.base.x + offset.x;
    camera_transform.translation.y = camera_rig.base.y + offset.y;
}
