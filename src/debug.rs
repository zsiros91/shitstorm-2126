use bevy::log::info;
use bevy::math::Isometry2d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::arena::{Arena, Tile};
use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::enemy::{spawn_enemy, Enemy, EnemyKind};
use crate::player::Player;
use crate::states::GameState;
use crate::waves::{WaveManager, WavePhase};

#[derive(Resource, Default)]
pub struct DebugFlags {
    pub show_colliders: bool,
    pub god_mode: bool,
    pub slow_mo: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugFlags>()
            .add_systems(Update, auto_start)
            .add_systems(
                Update,
                (debug_keys, debug_draw, debug_status, auto_screenshot)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut flags: ResMut<DebugFlags>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    arena: Res<Arena>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    mut waves: Option<ResMut<WaveManager>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        flags.show_colliders = !flags.show_colliders;
    }

    if keys.just_pressed(KeyCode::F2) {
        if let Ok(player_transform) = player.single() {
            let wave = waves.as_deref().map(|waves| waves.wave).unwrap_or(1);
            let limit = arena.half - Vec2::splat(50.0);
            for _ in 0..10 {
                let angle = fastrand::f32() * std::f32::consts::TAU;
                let distance = 180.0 + fastrand::f32() * 220.0;
                let position = (player_transform.translation.xy()
                    + Vec2::from_angle(angle) * distance)
                    .clamp(-limit, limit);
                let kind = match fastrand::u32(0..3) {
                    0 => EnemyKind::Walker,
                    1 => EnemyKind::Runner,
                    _ => EnemyKind::Brute,
                };
                spawn_enemy(&mut commands, &assets, kind, position, wave);
            }
        }
    }

    if keys.just_pressed(KeyCode::F3) {
        for entity in &enemies {
            commands.entity(entity).despawn();
        }
    }

    if keys.just_pressed(KeyCode::F4) {
        flags.slow_mo = !flags.slow_mo;
        virtual_time.set_relative_speed(if flags.slow_mo { 0.25 } else { 1.0 });
    }

    if keys.just_pressed(KeyCode::F5) {
        flags.god_mode = !flags.god_mode;
    }

    if keys.just_pressed(KeyCode::F6) {
        if let Some(waves) = waves.as_mut() {
            for entity in &enemies {
                commands.entity(entity).despawn();
            }
            waves.remaining_to_spawn = 0;
            waves.phase = WavePhase::Clearing;
        }
    }

    if keys.just_pressed(KeyCode::F7) {
        next_state.set(GameState::GameOver);
    }
}

fn debug_status(
    time: Res<Time>,
    mut timer: Local<f32>,
    player: Query<&Transform, With<Player>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    projection: Query<&Projection, With<Camera2d>>,
    tiles: Query<&Transform, With<Tile>>,
    assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    server: Res<AssetServer>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    info!("player={:?}", player.single().map(|t| t.translation));
    info!("camera={:?}", camera.single().map(|(_, t)| t.translation()));
    info!("tiles={}", tiles.iter().count());

    let tile_loaded: Vec<bool> = assets
        .tiles
        .iter()
        .map(|handle| images.get(handle).is_some())
        .collect();
    let tile_states: Vec<Option<bevy::asset::LoadState>> = assets
        .tiles
        .iter()
        .map(|handle| server.get_load_state(handle))
        .collect();
    let tile_sizes: Vec<Option<UVec2>> = assets
        .tiles
        .iter()
        .map(|handle| images.get(handle).map(|image| image.size()))
        .collect();
    info!("tile_loaded={:?}", tile_loaded);
    info!("tile_states={:?}", tile_states);
    info!("tile_sizes={:?}", tile_sizes);
    info!(
        "hull_loaded={:?} turret_loaded={:?} bullet_loaded={:?}",
        images.get(&assets.player_hull).map(|image| image.size()),
        images.get(&assets.player_turret).map(|image| image.size()),
        images.get(&assets.bullet).map(|image| image.size()),
    );

    if let (Ok(player_transform), Ok((camera, camera_transform))) =
        (player.single(), camera.single())
    {
        let ndc = camera.world_to_ndc(camera_transform, player_transform.translation);
        let viewport = camera
            .world_to_viewport(camera_transform, player_transform.translation)
            .ok();
        info!("player_ndc={:?} player_viewport={:?}", ndc, viewport);
    }

    if let Ok(projection) = projection.single() {
        if let Projection::Orthographic(ortho) = projection {
            info!(
                "ortho near={} far={} scale={} viewport_origin={:?}",
                ortho.near, ortho.far, ortho.scale, ortho.viewport_origin
            );
        }
    }
}

fn auto_start(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    if *state.get() != GameState::Menu {
        return;
    }
    *timer += time.delta_secs();
    if *timer >= 2.0 {
        *timer = 0.0;
        info!("auto_start: entering Playing");
        next_state.set(GameState::Playing);
    }
}

fn auto_screenshot(
    time: Res<Time>,
    mut commands: Commands,
    mut timer: Local<f32>,
    mut taken: Local<bool>,
) {
    if *taken {
        return;
    }
    *timer += time.delta_secs();
    if *timer < 5.0 {
        return;
    }
    *taken = true;
    let path = "C:/Users/User/AppData/Local/Temp/opencode/bevy_shot.png";
    info!("auto_screenshot: saving {path}");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

fn debug_draw(
    flags: Res<DebugFlags>,
    mut gizmos: Gizmos,
    colliders: Query<(&Transform, &Collider)>,
    statics: Query<(&Transform, &StaticCollider)>,
) {
    if !flags.show_colliders {
        return;
    }

    for (transform, collider) in &colliders {
        let center = transform.translation.xy();
        match collider {
            Collider::Circle { radius } => {
                gizmos.circle_2d(
                    Isometry2d::from_translation(center),
                    *radius,
                    Color::srgb(0.2, 1.0, 0.3),
                );
            }
            Collider::Aabb { half } => {
                draw_aabb(&mut gizmos, center, *half, Color::srgb(0.2, 1.0, 0.3));
            }
        }
    }

    for (transform, static_collider) in &statics {
        let center = transform.translation.xy();
        match static_collider.0 {
            Collider::Circle { radius } => {
                gizmos.circle_2d(
                    Isometry2d::from_translation(center),
                    radius,
                    Color::srgb(1.0, 0.85, 0.2),
                );
            }
            Collider::Aabb { half } => {
                draw_aabb(&mut gizmos, center, half, Color::srgb(1.0, 0.85, 0.2));
            }
        }
    }
}

fn draw_aabb(gizmos: &mut Gizmos, center: Vec2, half: Vec2, color: Color) {
    let bottom_left = center + Vec2::new(-half.x, -half.y);
    let bottom_right = center + Vec2::new(half.x, -half.y);
    let top_right = center + Vec2::new(half.x, half.y);
    let top_left = center + Vec2::new(-half.x, half.y);

    gizmos.line_2d(bottom_left, bottom_right, color);
    gizmos.line_2d(bottom_right, top_right, color);
    gizmos.line_2d(top_right, top_left, color);
    gizmos.line_2d(top_left, bottom_left, color);
}
