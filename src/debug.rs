use bevy::log::info;
use bevy::math::Isometry2d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::arena::Arena;
use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::enemy::{spawn_enemy, Enemy, EnemyKind};
use crate::player::Player;
use crate::states::GameState;
use crate::tower::{spawn_tower, TowerKind};
use crate::waves::{WaveManager, WavePhase};

#[derive(Resource, Default)]
pub struct DebugFlags {
    pub show_colliders: bool,
    pub god_mode: bool,
    pub slow_mo: bool,
    pub auto_start: bool,
    pub demo_towers: bool,
    pub screenshot: Option<(String, f32)>,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugFlags>()
            .add_systems(Startup, read_env_flags)
            .add_systems(
                OnEnter(GameState::Playing),
                seed_demo_towers.run_if(demo_towers_enabled),
            )
            .add_systems(Update, auto_start.run_if(in_state(GameState::Menu)))
            .add_systems(Update, auto_screenshot)
            .add_systems(
                Update,
                (debug_keys, debug_draw, debug_status).run_if(in_state(GameState::Playing)),
            );
    }
}

fn demo_towers_enabled(flags: Res<DebugFlags>) -> bool {
    flags.demo_towers
}

fn seed_demo_towers(mut commands: Commands, assets: Res<GameAssets>) {
    for (kind, position) in [
        (TowerKind::Gauss, Vec2::new(-220.0, -110.0)),
        (TowerKind::Missile, Vec2::new(220.0, -110.0)),
        (TowerKind::Relay, Vec2::new(-360.0, 130.0)),
        (TowerKind::Arc, Vec2::new(0.0, 190.0)),
    ] {
        spawn_tower(&mut commands, &assets, kind, position);
    }
}

fn read_env_flags(mut flags: ResMut<DebugFlags>) {
    flags.auto_start = std::env::var("SHITSTORM_AUTOSTART").is_ok();
    flags.demo_towers = std::env::var("SHITSTORM_DEMO_TOWERS").is_ok();
    if let Ok(path) = std::env::var("SHITSTORM_SCREENSHOT") {
        let delay = std::env::var("SHITSTORM_SCREENSHOT_DELAY")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(5.0);
        flags.screenshot = Some((path, delay));
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
                    0 => EnemyKind::Zergling,
                    1 => EnemyKind::Hydralisk,
                    _ => EnemyKind::Ultralisk,
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
        info!("god_mode={}", flags.god_mode);
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

    if keys.just_pressed(KeyCode::F12) {
        let path = std::env::var("SHITSTORM_SCREENSHOT")
            .unwrap_or_else(|_| "/tmp/opencode/bevy_shot.png".to_string());
        info!("saving screenshot to {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn debug_status(
    time: Res<Time>,
    mut timer: Local<f32>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<&Enemy>,
    towers: Query<&crate::tower::Tower>,
    images: Res<Assets<Image>>,
    assets: Res<GameAssets>,
) {
    *timer += time.delta_secs();
    if *timer < 2.0 {
        return;
    }
    *timer = 0.0;

    let player_position = player
        .single()
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);
    info!(
        "player={:?} enemies={} towers={} sprites_ready={}",
        player_position,
        enemies.iter().count(),
        towers.iter().count(),
        images.get(&assets.player_torso).is_some()
    );
}

fn auto_start(
    mut flags: ResMut<DebugFlags>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    if !flags.auto_start {
        return;
    }
    *timer += time.delta_secs();
    if *timer >= 1.0 {
        *timer = 0.0;
        flags.auto_start = false;
        info!("auto_start: entering Playing");
        next_state.set(GameState::Playing);
    }
}

fn auto_screenshot(
    time: Res<Time>,
    mut commands: Commands,
    mut flags: ResMut<DebugFlags>,
    mut timer: Local<f32>,
) {
    let Some((path, delay)) = flags.screenshot.clone() else {
        return;
    };
    *timer += time.delta_secs();
    if *timer < delay {
        return;
    }
    info!("auto_screenshot: saving {path}");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    flags.screenshot = None;
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
