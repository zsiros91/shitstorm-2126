use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::arena::{PLAYER_SPAWN, Z_LIGHT, y_sort};
use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider, circle_aabb_pushout};
use crate::combat::{Health, HitFlash};
use crate::fx::spawn_dust;
use crate::states::{GameSet, GameState, gameplay_active};
use crate::weapon::Weapon;

pub const PLAYER_RADIUS: f32 = 9.0;
const WALK_SPEED: f32 = 112.0;
const SPRINT_SPEED: f32 = 178.0;
const STAMINA_MAX: f32 = 100.0;
const STAMINA_DRAIN: f32 = 30.0;
const STAMINA_REGEN: f32 = 22.0;

#[derive(Component)]
pub struct Player;

/// Shared 4-direction walk-cycle state (0=down 1=left 2=up 3=right).
#[derive(Component)]
pub struct WalkAnim {
    pub timer: f32,
    pub frame: usize,
    pub dir: usize,
}

impl Default for WalkAnim {
    fn default() -> Self {
        Self {
            timer: 0.0,
            frame: 0,
            dir: 0,
        }
    }
}

#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub delay: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: STAMINA_MAX,
            max: STAMINA_MAX,
            delay: 0.0,
        }
    }
}

/// Movement intent written by the input system each frame.
#[derive(Component, Default)]
pub struct MoveIntent {
    pub dir: Vec2,
    pub sprint: bool,
}

/// Mouse aim written by the input system each frame.
#[derive(Component, Default)]
pub struct Aim {
    pub point: Vec2,
    pub angle: f32,
}

/// Smoothed world velocity, so acceleration stays frame-rate independent.
#[derive(Component, Default)]
pub struct MoveVelocity(pub Vec2);

#[derive(Component)]
pub struct PlayerGun;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (player_input, player_move, player_aim_visual)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(gameplay_active),
            );
    }
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    let position = PLAYER_SPAWN;
    commands
        .spawn((
            Sprite {
                image: assets.player_body[0][0].clone(),
                custom_size: Some(Vec2::new(24.0, 36.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, y_sort(position.y)),
            Player,
            Health::new(100.0),
            HitFlash::default(),
            Collider::circle(PLAYER_RADIUS),
            WalkAnim::default(),
            Stamina::default(),
            MoveIntent::default(),
            Aim::default(),
            MoveVelocity::default(),
            Weapon::rifle(),
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: assets.shadow_small.clone(),
                    custom_size: Some(Vec2::new(22.0, 14.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                    ..default()
                },
                Transform::from_xyz(2.0, -6.0, -1.4),
            ));
            parent.spawn((
                Sprite {
                    image: assets.player_gun.clone(),
                    ..default()
                },
                Transform::from_xyz(7.0, 0.0, 0.2),
                PlayerGun,
            ));
        });

    // the survivor carries a small cool lantern light
    commands.spawn((
        Sprite {
            image: assets.glow_cool.clone(),
            custom_size: Some(Vec2::splat(230.0)),
            color: Color::srgba(0.72, 0.82, 1.0, 0.34),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, Z_LIGHT),
        PlayerLight,
        DespawnOnExit(GameState::Playing),
    ));
}

#[derive(Component)]
pub struct PlayerLight;

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut player: Query<(&Transform, &mut MoveIntent, &mut Aim), With<Player>>,
) {
    let Ok((transform, mut intent, mut aim)) = player.single_mut() else {
        return;
    };

    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    intent.dir = dir.normalize_or_zero();
    intent.sprint = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    let (camera, camera_transform) = *camera;
    if let Some(cursor) = window.cursor_position() {
        if let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor) {
            aim.point = point;
            let delta = point - transform.translation.xy();
            if delta.length_squared() > 1.0 {
                aim.angle = delta.y.atan2(delta.x);
            }
        }
    }
}

fn player_move(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut player: Query<
        (
            &mut Transform,
            &mut Sprite,
            &mut WalkAnim,
            &mut Stamina,
            &mut MoveVelocity,
            &MoveIntent,
            &Collider,
        ),
        With<Player>,
    >,
    statics: Query<(&Transform, &StaticCollider), Without<Player>>,
    mut dust_timer: Local<f32>,
) {
    let dt = time.delta_secs();
    let Ok((mut transform, mut sprite, mut walk, mut stamina, mut velocity, intent, collider)) =
        player.single_mut()
    else {
        return;
    };

    let moving = intent.dir.length_squared() > 0.0;
    let sprinting = intent.sprint && moving && stamina.current > 0.0;

    if sprinting {
        stamina.current = (stamina.current - STAMINA_DRAIN * dt).max(0.0);
        stamina.delay = 0.7;
    } else {
        stamina.delay = (stamina.delay - dt).max(0.0);
        if stamina.delay <= 0.0 {
            stamina.current = (stamina.current + STAMINA_REGEN * dt).min(stamina.max);
        }
    }

    let target_speed = if sprinting { SPRINT_SPEED } else { WALK_SPEED };
    let desired = intent.dir * target_speed;
    let response = if sprinting { 9.0 } else { 12.0 };
    let blend = 1.0 - (-response * dt).exp();
    velocity.0 = velocity.0.lerp(desired, blend);
    let velocity = velocity.0;

    let mut next = transform.translation.xy() + velocity * dt;
    let radius = collider.radius();

    for (static_transform, static_collider) in &statics {
        match static_collider.0 {
            Collider::Circle {
                radius: static_radius,
            } => {
                let delta = next - static_transform.translation.xy();
                let min_distance = static_radius + radius;
                let distance_sq = delta.length_squared();
                if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                    next += delta.normalize() * (min_distance - distance_sq.sqrt());
                }
            }
            Collider::Aabb { half } => {
                if let Some(push) =
                    circle_aabb_pushout(next, radius, static_transform.translation.xy(), half)
                {
                    next += push;
                }
            }
        }
    }

    let arena_half = crate::arena::HALF - Vec2::splat(radius + 4.0);
    next.x = next.x.clamp(-arena_half.x, arena_half.x);
    next.y = next.y.clamp(-arena_half.y, arena_half.y);
    transform.translation.x = next.x;
    transform.translation.y = next.y;
    transform.translation.z = y_sort(next.y);

    // footfall dust while sprinting
    if sprinting {
        *dust_timer -= dt;
        if *dust_timer <= 0.0 {
            *dust_timer = 0.14;
            spawn_dust(&mut commands, &assets, next - intent.dir * 8.0);
        }
    } else {
        *dust_timer = 0.0;
    }

    // walk cycle + facing
    let speed = velocity.length();
    if speed > 8.0 {
        if velocity.x.abs() > velocity.y.abs() {
            walk.dir = if velocity.x >= 0.0 { 3 } else { 1 };
        } else {
            walk.dir = if velocity.y >= 0.0 { 2 } else { 0 };
        }
        walk.timer += dt;
        let step = if sprinting { 0.085 } else { 0.12 };
        if walk.timer >= step {
            walk.timer = 0.0;
            walk.frame = (walk.frame + 1) % 2;
        }
    } else {
        walk.timer = 0.0;
        walk.frame = 0;
    }
    sprite.image = assets.player_body[walk.dir][walk.frame].clone();
    sprite.custom_size = Some(Vec2::new(24.0, 36.0));
}

fn player_aim_visual(
    mut player: Query<(&Transform, &mut WalkAnim, &mut Sprite, &Aim, &MoveIntent), With<Player>>,
    mut guns: Query<&mut Transform, (With<PlayerGun>, Without<Player>)>,
    mut lights: Query<&mut Transform, (With<PlayerLight>, Without<PlayerGun>, Without<Player>)>,
    assets: Res<GameAssets>,
) {
    let Ok((transform, mut walk, mut sprite, aim, intent)) = player.single_mut() else {
        return;
    };
    let position = transform.translation;

    // when stationary the survivor faces the crosshair instead of the step
    if intent.dir.length_squared() < 0.01 {
        let a = aim.angle;
        walk.dir = if a.abs() < std::f32::consts::FRAC_PI_4 {
            3
        } else if a > std::f32::consts::FRAC_PI_4 && a < 3.0 * std::f32::consts::FRAC_PI_4 {
            2
        } else if a < -std::f32::consts::FRAC_PI_4 && a > -3.0 * std::f32::consts::FRAC_PI_4 {
            0
        } else {
            1
        };
        sprite.image = assets.player_body[walk.dir][walk.frame].clone();
    }

    if let Ok(mut gun) = guns.single_mut() {
        gun.translation.x = aim.angle.cos() * 7.0;
        gun.translation.y = aim.angle.sin() * 7.0;
        gun.rotation = Quat::from_rotation_z(aim.angle);
    }

    if let Ok(mut light) = lights.single_mut() {
        // the pool tracks the survivor on its own layer so it never
        // inherits y-sorting from the body
        light.translation.x = position.x;
        light.translation.y = position.y;
        light.translation.z = Z_LIGHT;
    }
}
