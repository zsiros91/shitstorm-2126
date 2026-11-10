use bevy::log::info;
use bevy::math::Isometry2d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;

use crate::arena::Arena;
use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::enemy::{Enemy, EnemyKind, spawn_enemy};
use crate::player::Player;
use crate::states::{GameSet, GameState};

#[derive(Resource, Default)]
pub struct DebugFlags {
    pub show_colliders: bool,
    pub god_mode: bool,
    pub slow_mo: bool,
    pub auto_start: bool,
    /// Scripted walk + fire sequence (`SHITSTORM_DEMO_INPUT`), used to
    /// capture gameplay screenshots without a real mouse or keyboard.
    pub demo_input: bool,
    pub screenshot: Option<(String, f32)>,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugFlags>()
            .add_systems(Startup, read_env_flags)
            .add_systems(Update, auto_start.run_if(in_state(GameState::Menu)))
            .add_systems(Update, auto_screenshot)
            .add_systems(
                Update,
                demo_input
                    .before(GameSet::Input)
                    .run_if(demo_enabled)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (debug_keys, debug_draw, debug_status).run_if(in_state(GameState::Playing)),
            );
    }
}

fn demo_enabled(flags: Res<DebugFlags>) -> bool {
    flags.demo_input
}

fn read_env_flags(mut flags: ResMut<DebugFlags>) {
    flags.auto_start = std::env::var("SHITSTORM_AUTOSTART").is_ok();
    flags.demo_input = std::env::var("SHITSTORM_DEMO_INPUT").is_ok();
    if flags.demo_input {
        // keep the scripted run alive long enough to capture the shot
        flags.god_mode = true;
    }
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
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::F1) {
        flags.show_colliders = !flags.show_colliders;
    }

    // F2: ring of mixed hostiles around the player
    if keys.just_pressed(KeyCode::F2) {
        spawn_hostile_ring(&mut commands, &assets, &arena, &player);
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

/// Ring of mixed hostiles dropped around the survivor (F2 and demo runs).
fn spawn_hostile_ring(
    commands: &mut Commands,
    assets: &GameAssets,
    arena: &Arena,
    player: &Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let limit = arena.half - Vec2::splat(50.0);
    for i in 0..10 {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        let distance = 160.0 + fastrand::f32() * 200.0;
        let position = (player_transform.translation.xy() + Vec2::from_angle(angle) * distance)
            .clamp(-limit, limit);
        let kind = if i % 3 == 0 {
            EnemyKind::Raider
        } else {
            EnemyKind::Infected
        };
        spawn_enemy(commands, assets, kind, position);
    }
}

/// Scripted input for `SHITSTORM_DEMO_INPUT`: seeds a hostile ring, then
/// walks the survivor and fires in bursts so a single screenshot shows
/// movement, muzzle flashes, the magazine gauge and the reload strip.
fn demo_input(
    time: Res<Time>,
    mut elapsed: Local<f32>,
    mut seeded: Local<bool>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), Without<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    arena: Res<Arena>,
) {
    *elapsed += time.delta_secs();
    let t = *elapsed;

    if !*seeded && !player.is_empty() {
        *seeded = true;
        spawn_hostile_ring(&mut commands, &assets, &arena, &player);
    }

    // aim at the nearest hostile that is not behind a HUD panel, so the
    // scripted shot stays exactly on target (no clamping to dodge panels,
    // which would skew the aim badly at contact range)
    let size = window.size();
    let mut aim_point = None;
    if let Ok(player_transform) = player.single() {
        let (camera, camera_transform) = *camera;
        let player_position = player_transform.translation.xy();
        let mut best_clear: Option<(f32, Vec2)> = None;
        let mut nearest_any: Option<(f32, Vec2)> = None;
        for foe in enemies.iter() {
            let world = foe.translation.xy();
            let distance = world.distance_squared(player_position);
            if nearest_any.is_none_or(|(best, _)| distance < best) {
                nearest_any = Some((distance, world));
            }
            if let Ok(point) = camera.world_to_viewport(camera_transform, foe.translation) {
                if !crate::ui::hud_blocks(point, size)
                    && best_clear.is_none_or(|(best, _)| distance < best)
                {
                    best_clear = Some((distance, point));
                }
            }
        }
        aim_point = best_clear.map(|(_, point)| point).or_else(|| {
            nearest_any.and_then(|(_, world)| {
                camera
                    .world_to_viewport(camera_transform, world.extend(0.0))
                    .ok()
            })
        });
    }
    window.set_cursor_position(Some(
        aim_point.unwrap_or(Vec2::new(size.x * 0.6, size.y * 0.4)),
    ));

    // walk north, cut east, then stand and shoot
    let step = if t < 2.0 {
        Some(KeyCode::KeyW)
    } else if t < 3.6 {
        Some(KeyCode::KeyD)
    } else {
        None
    };
    for key in [KeyCode::KeyW, KeyCode::KeyD] {
        if step == Some(key) {
            keys.press(key);
        } else {
            keys.release(key);
        }
    }

    // flash the collider overlay for one frame so screenshots can show it
    if (1.0..1.15).contains(&t) {
        keys.press(KeyCode::F1);
    } else {
        keys.release(KeyCode::F1);
    }

    // one long burst: the magazine runs dry and the reload strip shows
    let firing = (4.2..7.4).contains(&t);
    if firing {
        mouse.press(MouseButton::Left);
    } else {
        mouse.release(MouseButton::Left);
    }
}

fn debug_status(
    time: Res<Time>,
    mut timer: Local<f32>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<&Enemy>,
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
        "player={:?} enemies={} sprites_ready={}",
        player_position,
        enemies.iter().count(),
        images.get(&assets.player_body[0][0]).is_some()
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
