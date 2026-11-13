use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::PrimaryWindow;

use crate::arena::{PLAYER_SPAWN, y_sort};
use crate::assets::GameAssets;
use crate::camera::OffscreenRenderCamera;
use crate::collision::{Collider, StaticCollider, move_circle_with_collisions};
use crate::combat::{Health, HitFlash};
use crate::fx::spawn_dust;
use crate::states::{GameSet, GameState, gameplay_active};
use crate::weapon::Weapon;

pub const PLAYER_RADIUS: f32 = 9.0;
const PLAYER_SIZE: Vec2 = Vec2::new(38.0, 70.0);
const GUN_RENDER_SIZE: u32 = 40;
const GUN_RENDER_LAYER: usize = 1;
const WALK_SPEED: f32 = 112.0;
const SPRINT_SPEED: f32 = 178.0;
const STAMINA_MAX: f32 = 100.0;
const STAMINA_DRAIN: f32 = 30.0;
const STAMINA_REGEN: f32 = 22.0;

#[derive(Component)]
pub struct Player;

/// Walk-cycle state; enemies also use `dir` for their four facing directions.
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

#[derive(Component)]
struct GunRenderSource;

#[derive(Resource)]
struct GunRenderTarget(Handle<Image>);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gun_render_target)
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (player_input, player_move, player_aim_visual)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(gameplay_active),
            );
    }
}

fn setup_gun_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    assets: Res<GameAssets>,
) {
    let target = images.add(Image::new_target_texture(
        GUN_RENDER_SIZE,
        GUN_RENDER_SIZE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.insert_resource(GunRenderTarget(target.clone()));

    commands.spawn((
        Sprite::from_image(assets.player_gun.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GunRenderSource,
        RenderLayers::layer(GUN_RENDER_LAYER),
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        RenderTarget::Image(target.into()),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, 0.0, 100.0),
        Msaa::Off,
        RenderLayers::layer(GUN_RENDER_LAYER),
        OffscreenRenderCamera,
    ));
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<GameAssets>,
    gun_render_target: Res<GunRenderTarget>,
) {
    let position = PLAYER_SPAWN;
    commands
        .spawn((
            Sprite {
                image: assets.player_body[0][0].clone(),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            Transform::from_xyz(
                position.x,
                position.y,
                y_sort(position.y - PLAYER_SIZE.y * 0.5),
            ),
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
                Transform::from_xyz(2.0, -PLAYER_SIZE.y * 0.5 + 6.0, -1.4),
            ));
            parent.spawn((
                Sprite {
                    image: gun_render_target.0.clone(),
                    custom_size: Some(Vec2::splat(GUN_RENDER_SIZE as f32)),
                    ..default()
                },
                Transform::from_xyz(7.0, 0.0, 0.2),
                PlayerGun,
            ));
        });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), Without<OffscreenRenderCamera>>,
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
    mut static_colliders: Local<Vec<(Vec2, Collider)>>,
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

    static_colliders.clear();
    static_colliders.extend(
        statics
            .iter()
            .map(|(static_transform, collider)| (static_transform.translation.xy(), collider.0)),
    );
    let radius = collider.radius();
    let mut next = move_circle_with_collisions(
        transform.translation.xy(),
        velocity * dt,
        radius,
        &static_colliders,
    );

    let arena_half = crate::arena::HALF - Vec2::splat(radius + 4.0);
    next.x = next.x.clamp(-arena_half.x, arena_half.x);
    next.y = next.y.clamp(-arena_half.y, arena_half.y);
    transform.translation.x = next.x;
    transform.translation.y = next.y;
    transform.translation.z = y_sort(next.y - PLAYER_SIZE.y * 0.5);

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

    // Keep the walk cycle independent of facing: the player always uses the
    // front-facing frames, and aim controls its horizontal mirroring.
    let speed = velocity.length();
    if speed > 8.0 {
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
    sprite.image = assets.player_body[0][walk.frame].clone();
    sprite.custom_size = Some(PLAYER_SIZE);
}

fn player_aim_visual(
    mut player: Query<(&WalkAnim, &mut Sprite, &Aim), With<Player>>,
    mut guns: Query<&mut Transform, (With<PlayerGun>, Without<GunRenderSource>, Without<Player>)>,
    mut gun_source: Query<
        (&mut Sprite, &mut Transform),
        (With<GunRenderSource>, Without<PlayerGun>, Without<Player>),
    >,
    assets: Res<GameAssets>,
) {
    let Ok((walk, mut sprite, aim)) = player.single_mut() else {
        return;
    };

    let looking_left = aim.angle.cos() < -0.001;
    sprite.image = assets.player_body[0][walk.frame].clone();
    sprite.flip_x = looking_left;

    if let Ok(mut gun) = guns.single_mut() {
        gun.translation.x = (aim.angle.cos() * 7.0).round();
        gun.translation.y = (aim.angle.sin() * 7.0).round();
        gun.translation.z = 0.2;
        gun.rotation = Quat::IDENTITY;
    }

    if let Ok((mut source_sprite, mut source_transform)) = gun_source.single_mut() {
        source_sprite.flip_x = looking_left;
        let render_angle = if looking_left {
            aim.angle + std::f32::consts::PI
        } else {
            aim.angle
        };
        source_transform.rotation = Quat::from_rotation_z(render_angle);
    }
}
