use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::arena::{Z_BRIGHT, Z_DECAL};
use crate::assets::GameAssets;
use crate::camera::OffscreenRenderCamera;
use crate::combat::EnemyKilled;
use crate::states::{GameSet, GameState, gameplay_active};

const TRACER_RENDER_SIZE: u32 = 64;
const CASING_RENDER_SIZE: u32 = 8;

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub drag: f32,
}

#[derive(Component)]
pub struct FadeOut {
    pub timer: f32,
    pub duration: f32,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>().add_systems(
            Update,
            (enemy_death_fx, update_particles, fade_effects)
                .chain()
                .in_set(GameSet::Effects)
                .run_if(gameplay_active),
        );
    }
}

pub fn spawn_burst(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec2,
    color: Color,
    count: usize,
    speed: f32,
    size: f32,
) {
    for _ in 0..count {
        let dir = Vec2::from_angle(fastrand::f32() * std::f32::consts::TAU);
        let spd = speed * (0.25 + fastrand::f32() * 0.75);
        let life = 0.15 + fastrand::f32() * 0.3;
        commands.spawn((
            Sprite {
                image: assets.spark.clone(),
                custom_size: Some(Vec2::splat(size)),
                color,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, Z_BRIGHT),
            Particle {
                velocity: dir * spd,
                drag: 5.0,
            },
            FadeOut {
                timer: life,
                duration: life,
            },
            DespawnOnExit(GameState::Playing),
        ));
    }
}

pub fn spawn_impact(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    spawn_burst(
        commands,
        assets,
        position,
        Color::srgb(1.0, 0.82, 0.35),
        4,
        160.0,
        4.0,
    );
}

/// Red droplets + a lingering ground pool.
pub fn spawn_blood(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    spawn_burst(
        commands,
        assets,
        position,
        Color::srgb(0.72, 0.14, 0.12),
        10,
        150.0,
        5.0,
    );

    commands.spawn((
        Sprite {
            image: assets.blood_splat.clone(),
            custom_size: Some(Vec2::splat(28.0 + fastrand::f32() * 16.0)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.95),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, Z_DECAL).with_rotation(Quat::from_rotation_z(
            fastrand::f32() * std::f32::consts::TAU,
        )),
        FadeOut {
            timer: 24.0,
            duration: 24.0,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

/// Footfall dust puff for sprinting.
pub fn spawn_dust(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.dust_puff.clone(),
            custom_size: Some(Vec2::splat(14.0 + fastrand::f32() * 8.0)),
            color: Color::srgba(0.82, 0.76, 0.62, 0.55),
            ..default()
        },
        Transform::from_xyz(
            position.x + (fastrand::f32() - 0.5) * 6.0,
            position.y + (fastrand::f32() - 0.5) * 6.0,
            Z_BRIGHT - 2.0,
        ),
        Particle {
            velocity: Vec2::from_angle(fastrand::f32() * std::f32::consts::TAU) * 14.0,
            drag: 3.0,
        },
        FadeOut {
            timer: 0.4,
            duration: 0.4,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn spawn_tracer(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    assets: &GameAssets,
    start: Vec2,
    end: Vec2,
    color: Color,
    width: f32,
    life: f32,
) {
    let delta = end - start;
    let length = delta.length();
    if length < 1.0 {
        return;
    }
    let angle = delta.y.atan2(delta.x);
    let mid = (start + end) * 0.5;
    let target = images.add(Image::new_target_texture(
        TRACER_RENDER_SIZE,
        TRACER_RENDER_SIZE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));

    let tracer = commands
        .spawn((
            Sprite {
                image: target.clone(),
                custom_size: Some(Vec2::splat(TRACER_RENDER_SIZE as f32)),
                color,
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, Z_BRIGHT + 1.0),
            FadeOut {
                timer: life,
                duration: life,
            },
            DespawnOnExit(GameState::Playing),
        ))
        .id();

    let render_layer = RenderLayers::from_layers(&[tracer.index_u32() as usize + 2]);
    commands.entity(tracer).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.tracer.clone(),
                custom_size: Some(Vec2::new(length, width)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1).with_rotation(Quat::from_rotation_z(angle)),
            render_layer.clone(),
        ));
        parent.spawn((
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
            render_layer,
            OffscreenRenderCamera,
        ));
    });
}

pub fn spawn_casing(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    casing: Handle<Image>,
    position: Vec2,
    rotation: f32,
    velocity: Vec2,
    life: f32,
) {
    let target = images.add(Image::new_target_texture(
        CASING_RENDER_SIZE,
        CASING_RENDER_SIZE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));

    let casing_entity = commands
        .spawn((
            Sprite {
                image: target.clone(),
                custom_size: Some(Vec2::splat(CASING_RENDER_SIZE as f32)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, Z_BRIGHT),
            Particle {
                velocity,
                drag: 4.0,
            },
            FadeOut {
                timer: life,
                duration: life,
            },
            DespawnOnExit(GameState::Playing),
        ))
        .id();

    let render_layer = RenderLayers::from_layers(&[casing_entity.index_u32() as usize + 2]);
    commands.entity(casing_entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: casing,
                custom_size: Some(Vec2::new(5.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1).with_rotation(Quat::from_rotation_z(rotation)),
            render_layer.clone(),
        ));
        parent.spawn((
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
            render_layer,
            OffscreenRenderCamera,
        ));
    });
}

fn enemy_death_fx(
    mut kills: MessageReader<EnemyKilled>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
) {
    for kill in kills.read() {
        spawn_burst(
            &mut commands,
            &assets,
            kill.position,
            Color::srgb(0.78, 0.16, 0.13),
            6,
            120.0,
            4.0,
        );
        shake.add(0.08);
    }
}

fn update_particles(time: Res<Time>, mut particles: Query<(&mut Transform, &mut Particle)>) {
    let dt = time.delta_secs();
    for (mut transform, mut particle) in &mut particles {
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        let drag = particle.drag;
        particle.velocity *= 1.0 - (drag * dt).min(1.0);
    }
}

fn fade_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut FadeOut, &mut Sprite)>,
) {
    for (entity, mut fade, mut sprite) in &mut effects {
        fade.timer -= time.delta_secs();
        let alpha = (fade.timer / fade.duration).clamp(0.0, 1.0);
        sprite.color.set_alpha(alpha);
        if fade.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
