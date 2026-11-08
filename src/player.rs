use bevy::math::EulerRot;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::{Arena, PLAYER_SPAWN};
use crate::assets::GameAssets;
use crate::build::BuildMode;
use crate::collision::{circle_aabb_pushout, Collider, StaticCollider};
use crate::combat::{Bullet, BulletKind, Health, HitFlash, Lifetime, Velocity};
use crate::fx::{spawn_burst, spawn_tracer, FadeOut, ScreenShake};
use crate::healthbar::spawn_health_bar;
use crate::states::{gameplay_active, GameSet, GameState};

const PLAYER_RADIUS: f32 = 27.0;
const PLAYER_ACCEL: f32 = 1900.0;
const PLAYER_MAX_SPEED: f32 = 285.0;
const FIRE_INTERVAL: f32 = 0.13;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerTurret;

#[derive(Component)]
pub struct PlayerLegs {
    pub timer: f32,
    pub frame: usize,
}

#[derive(Component)]
pub struct Crosshair;

#[derive(Component, Clone, Copy, Debug)]
pub struct Repair {
    pub charges: u8,
    pub max_charges: u8,
    pub channel: f32,
    pub channel_time: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (aim_turret, update_crosshair, player_shoot, player_repair)
                    .in_set(GameSet::Input)
                    .run_if(gameplay_active),
            )
            .add_systems(
                Update,
                (player_movement, animate_legs)
                    .in_set(GameSet::Movement)
                    .run_if(gameplay_active),
            );
    }
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    let spawn = PLAYER_SPAWN;

    let player = commands
        .spawn((
            Sprite {
                image: assets.player_legs[0].clone(),
                custom_size: Some(Vec2::splat(82.0)),
                ..default()
            },
            Transform::from_xyz(spawn.x, spawn.y, 10.0),
            Player,
            PlayerLegs {
                timer: 0.0,
                frame: 0,
            },
            Health::new(150.0),
            Velocity::default(),
            Collider::circle(PLAYER_RADIUS),
            HitFlash::default(),
            Repair {
                charges: 3,
                max_charges: 3,
                channel: 0.0,
                channel_time: 0.85,
            },
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: assets.shadow.clone(),
                    custom_size: Some(Vec2::splat(86.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.65),
                    ..default()
                },
                Transform::from_xyz(6.0, -8.0, -7.0),
            ));
            parent.spawn((
                Sprite {
                    image: assets.selection_ring.clone(),
                    custom_size: Some(Vec2::splat(84.0)),
                    color: Color::srgba(0.45, 1.0, 0.6, 0.55),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -7.5),
            ));
        })
        .id();

    commands.spawn((
        Sprite {
            image: assets.player_torso.clone(),
            custom_size: Some(Vec2::splat(82.0)),
            ..default()
        },
        Transform::from_xyz(spawn.x, spawn.y, 11.0),
        PlayerTurret,
        DespawnOnExit(GameState::Playing),
    ));

    commands.spawn((
        Sprite {
            image: assets.ring.clone(),
            custom_size: Some(Vec2::splat(24.0)),
            color: Color::srgba(0.45, 0.95, 1.0, 0.9),
            ..default()
        },
        Transform::from_xyz(spawn.x, spawn.y, 40.0),
        Crosshair,
        DespawnOnExit(GameState::Playing),
    ));

    spawn_health_bar(&mut commands, player, 74.0, 58.0, false);
}

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    arena: Res<Arena>,
    statics: Query<(&Transform, &StaticCollider), Without<Player>>,
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let Ok((mut transform, mut velocity)) = player.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    let mut axis = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        axis.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        axis.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        axis.x += 1.0;
    }
    let axis = axis.normalize_or_zero();

    velocity.0 += axis * PLAYER_ACCEL * dt;
    velocity.0 *= 1.0 - (8.0 * dt).min(1.0);
    velocity.0 = velocity.0.clamp_length_max(PLAYER_MAX_SPEED);

    transform.translation.x += velocity.0.x * dt;
    transform.translation.y += velocity.0.y * dt;

    for (static_transform, static_collider) in &statics {
        let position = transform.translation.xy();
        match static_collider.0 {
            Collider::Circle { radius } => {
                let delta = position - static_transform.translation.xy();
                let min_distance = radius + PLAYER_RADIUS;
                let distance_sq = delta.length_squared();
                if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                    let push = delta.normalize() * (min_distance - distance_sq.sqrt());
                    transform.translation.x += push.x;
                    transform.translation.y += push.y;
                }
            }
            Collider::Aabb { half } => {
                if let Some(push) = circle_aabb_pushout(
                    position,
                    PLAYER_RADIUS,
                    static_transform.translation.xy(),
                    half,
                ) {
                    transform.translation.x += push.x;
                    transform.translation.y += push.y;
                }
            }
        }
    }

    let limit = arena.half - Vec2::splat(PLAYER_RADIUS + 8.0);
    transform.translation.x = transform.translation.x.clamp(-limit.x, limit.x);
    transform.translation.y = transform.translation.y.clamp(-limit.y, limit.y);

    if velocity.0.length_squared() > 100.0 {
        let target = velocity.0.y.atan2(velocity.0.x) - std::f32::consts::FRAC_PI_2;
        transform.rotation = transform
            .rotation
            .slerp(Quat::from_rotation_z(target), (12.0 * dt).min(1.0));
    }
}

fn animate_legs(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut legs: Query<(&mut PlayerLegs, &mut Sprite, &Velocity)>,
) {
    let moving = legs
        .iter()
        .next()
        .map(|(_, _, velocity)| velocity.0.length_squared() > 400.0)
        .unwrap_or(false);

    for (mut walk, mut sprite, _) in &mut legs {
        if !moving {
            sprite.image = assets.player_legs[0].clone();
            walk.frame = 0;
            walk.timer = 0.0;
            continue;
        }
        walk.timer += time.delta_secs();
        if walk.timer >= 0.11 {
            walk.timer = 0.0;
            walk.frame = (walk.frame + 1) % 2;
            sprite.image = assets.player_legs[walk.frame].clone();
        }
    }
}

fn cursor_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn aim_turret(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Query<&Transform, (With<Player>, Without<PlayerTurret>)>,
    mut turret: Query<&mut Transform, (With<PlayerTurret>, Without<Player>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut turret_transform) = turret.single_mut() else {
        return;
    };
    let (camera, camera_transform) = *camera;

    let Some(cursor) = cursor_world_position(&window, camera, camera_transform) else {
        return;
    };

    let delta = cursor - player_transform.translation.xy();
    if delta.length_squared() < 1.0 {
        return;
    }

    let angle = delta.y.atan2(delta.x) - std::f32::consts::FRAC_PI_2;
    turret_transform.rotation = Quat::from_rotation_z(angle);
    turret_transform.translation.x = player_transform.translation.x;
    turret_transform.translation.y = player_transform.translation.y;
}

fn update_crosshair(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
    mut crosshair: Query<&mut Transform, (With<Crosshair>, Without<Player>)>,
) {
    let (camera, camera_transform) = *camera;
    let Some(cursor) = cursor_world_position(&window, camera, camera_transform) else {
        return;
    };
    let Ok(mut crosshair_transform) = crosshair.single_mut() else {
        return;
    };
    crosshair_transform.translation.x = cursor.x;
    crosshair_transform.translation.y = cursor.y;
    crosshair_transform.rotation =
        Quat::from_rotation_z(time.elapsed_secs() * 0.8);
}

fn player_shoot(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    build: Res<BuildMode>,
    window: Single<&Window, With<PrimaryWindow>>,
    player: Query<&Transform, With<Player>>,
    turret: Query<&Transform, (With<PlayerTurret>, Without<Player>)>,
    mut cooldown: Local<f32>,
    mut side: Local<bool>,
    mut shake: ResMut<ScreenShake>,
) {
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    if build.active
        || !mouse.pressed(MouseButton::Left)
        || *cooldown > 0.0
        || crate::ui::cursor_over_hud(&window)
    {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(turret_transform) = turret.single() else {
        return;
    };

    *cooldown = FIRE_INTERVAL;

    let angle = turret_transform.rotation.to_euler(EulerRot::XYZ).2;
    let spread = (fastrand::f32() - 0.5) * 0.035;
    let direction = Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2 + spread);
    let perpendicular = Vec2::new(-direction.y, direction.x);
    *side = !*side;
    let offset = if *side { 9.0 } else { -9.0 };
    let muzzle = player_transform.translation.xy() + direction * 38.0 + perpendicular * offset;

    commands.spawn((
        Sprite {
            image: assets.bullet_gauss.clone(),
            custom_size: Some(Vec2::new(10.0, 26.0)),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, 16.0)
            .with_rotation(Quat::from_rotation_z(angle + spread)),
        Velocity(direction * 1250.0),
        Bullet {
            damage: 17.0,
            radius: 6.0,
            splash: 0.0,
            slow: 0.0,
            kind: BulletKind::Gauss,
        },
        Lifetime(1.0),
        DespawnOnExit(GameState::Playing),
    ));

    spawn_tracer(
        &mut commands,
        &assets,
        muzzle,
        muzzle + direction * 90.0,
        Color::srgba(0.5, 0.95, 1.0, 0.9),
        2.4,
        0.06,
    );

    commands.spawn((
        Sprite {
            image: assets.muzzle_flash.clone(),
            custom_size: Some(Vec2::splat(34.0)),
            color: Color::srgb(1.0, 0.82, 0.4),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, 26.0)
            .with_rotation(Quat::from_rotation_z(angle + spread)),
        FadeOut {
            timer: 0.06,
            duration: 0.06,
        },
        DespawnOnExit(GameState::Playing),
    ));

    shake.add(0.045);
}

fn player_repair(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    build: Res<BuildMode>,
    mut heal_timer: Local<f32>,
    mut player: Query<(&Transform, &mut Health, &mut Repair, &Velocity), With<Player>>,
) {
    let Ok((transform, mut health, mut repair, velocity)) = player.single_mut() else {
        return;
    };

    let holding = keys.pressed(KeyCode::KeyR);
    let moving = velocity.0.length_squared() > 400.0;
    let firing = mouse.pressed(MouseButton::Left);

    if build.active || !holding || moving || firing || repair.charges == 0 || health.current >= health.max
    {
        repair.channel = 0.0;
        return;
    }

    repair.channel += time.delta_secs();
    *heal_timer += time.delta_secs();
    if *heal_timer >= 0.11 {
        *heal_timer = 0.0;
        spawn_burst(
            &mut commands,
            &assets,
            transform.translation.xy() + Vec2::new(0.0, 12.0),
            Color::srgb(0.3, 1.0, 0.45),
            2,
            55.0,
            4.0,
        );
    }

    if repair.channel >= repair.channel_time {
        repair.channel = 0.0;
        repair.charges -= 1;
        health.current = (health.current + 45.0).min(health.max);
        spawn_burst(
            &mut commands,
            &assets,
            transform.translation.xy(),
            Color::srgb(0.4, 1.0, 0.5),
            14,
            120.0,
            5.0,
        );
    }
}
