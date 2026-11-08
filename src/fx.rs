use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::combat::EnemyKilled;
use crate::states::{gameplay_active, GameSet, GameState};

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

#[derive(Component)]
pub struct ExplosionAnim {
    pub frames: Vec<Handle<Image>>,
    pub index: usize,
    pub timer: f32,
    pub frame_time: f32,
}

#[derive(Component)]
pub struct MaterializeAnim {
    pub timer: f32,
    pub duration: f32,
    pub start_scale: f32,
    pub end_scale: f32,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>().add_systems(
            Update,
            (
                enemy_death_fx,
                animate_explosions,
                animate_materialize,
                update_particles,
                fade_effects,
            )
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
            Transform::from_xyz(position.x, position.y, 28.0),
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

pub fn spawn_blood(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    spawn_burst(
        commands,
        assets,
        position,
        Color::srgb(0.62, 0.22, 0.75),
        9,
        150.0,
        5.0,
    );
    commands.spawn((
        Sprite {
            image: assets.creep_decal.clone(),
            custom_size: Some(Vec2::splat(60.0)),
            color: Color::srgba(0.8, 0.5, 1.0, 0.75),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, -8.0)
            .with_rotation(Quat::from_rotation_z(fastrand::f32() * std::f32::consts::TAU)),
        FadeOut {
            timer: 5.0,
            duration: 5.0,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn spawn_explosion(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.explosions[0].clone(),
            custom_size: Some(Vec2::splat(112.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 30.0),
        ExplosionAnim {
            frames: assets.explosions.to_vec(),
            index: 0,
            timer: 0.0,
            frame_time: 0.055,
        },
        DespawnOnExit(GameState::Playing),
    ));

    commands.spawn((
        Sprite {
            image: assets.smoke.clone(),
            custom_size: Some(Vec2::splat(70.0)),
            color: Color::srgba(0.55, 0.55, 0.58, 0.55),
            ..default()
        },
        Transform::from_xyz(position.x, position.y - 8.0, 22.0),
        FadeOut {
            timer: 1.1,
            duration: 1.1,
        },
        DespawnOnExit(GameState::Playing),
    ));

    commands.spawn((
        Sprite {
            image: assets.scorch.clone(),
            custom_size: Some(Vec2::splat(90.0)),
            color: Color::srgba(1.0, 1.0, 1.0, 0.8),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, -8.0)
            .with_rotation(Quat::from_rotation_z(fastrand::f32() * std::f32::consts::TAU)),
        FadeOut {
            timer: 9.0,
            duration: 9.0,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn spawn_tracer(
    commands: &mut Commands,
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

    commands.spawn((
        Sprite {
            image: assets.tracer.clone(),
            custom_size: Some(Vec2::new(length, width)),
            color,
            ..default()
        },
        Transform::from_xyz(mid.x, mid.y, 24.0).with_rotation(Quat::from_rotation_z(angle)),
        FadeOut {
            timer: life,
            duration: life,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn spawn_lightning(
    commands: &mut Commands,
    assets: &GameAssets,
    start: Vec2,
    end: Vec2,
    color: Color,
) {
    let delta = end - start;
    let segments = 5;
    let mut previous = start;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let point = start + delta * t
            + Vec2::new(
                (fastrand::f32() - 0.5) * 14.0 * (1.0 - t),
                (fastrand::f32() - 0.5) * 14.0 * (1.0 - t),
            );
        spawn_tracer(commands, assets, previous, point, color, 3.0, 0.11);
        previous = point;
    }
    spawn_tracer(commands, assets, start, end, Color::srgba(0.75, 0.95, 1.0, 0.6), 8.0, 0.09);
}

pub fn spawn_materialize(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.ring.clone(),
            custom_size: Some(Vec2::splat(60.0)),
            color: Color::srgba(0.35, 0.95, 1.0, 0.95),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 18.0),
        MaterializeAnim {
            timer: 0.0,
            duration: 0.55,
            start_scale: 0.35,
            end_scale: 1.8,
        },
        DespawnOnExit(GameState::Playing),
    ));

    commands.spawn((
        Sprite {
            image: assets.beam.clone(),
            custom_size: Some(Vec2::new(46.0, 260.0)),
            color: Color::srgba(0.5, 0.95, 1.0, 0.75),
            ..default()
        },
        Transform::from_xyz(position.x, position.y + 90.0, 40.0),
        FadeOut {
            timer: 0.45,
            duration: 0.45,
        },
        DespawnOnExit(GameState::Playing),
    ));

    spawn_burst(
        commands,
        assets,
        position,
        Color::srgb(0.45, 0.95, 1.0),
        14,
        120.0,
        4.0,
    );
}

pub fn spawn_selection_ring(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec2,
    size: f32,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: assets.selection_ring.clone(),
                custom_size: Some(Vec2::splat(size)),
                color: Color::srgba(0.6, 1.0, 0.7, 0.75),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 2.0),
            DespawnOnExit(GameState::Playing),
        ))
        .id()
}

fn enemy_death_fx(
    mut kills: MessageReader<EnemyKilled>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
) {
    for kill in kills.read() {
        spawn_explosion(&mut commands, &assets, kill.position);
        spawn_blood(&mut commands, &assets, kill.position);
        shake.add(0.13);
    }
}

fn animate_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut ExplosionAnim, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut explosions {
        anim.timer += time.delta_secs();
        while anim.timer >= anim.frame_time {
            anim.timer -= anim.frame_time;
            anim.index += 1;
            if anim.index >= anim.frames.len() {
                commands.entity(entity).despawn();
                break;
            }
            sprite.image = anim.frames[anim.index].clone();
            sprite.custom_size = Some(Vec2::splat(112.0));
        }
    }
}

fn animate_materialize(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut MaterializeAnim, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut anim, mut transform, mut sprite) in &mut effects {
        anim.timer += time.delta_secs();
        let t = (anim.timer / anim.duration).clamp(0.0, 1.0);
        let scale = anim.start_scale + (anim.end_scale - anim.start_scale) * t;
        sprite.custom_size = Some(Vec2::splat(60.0 * scale));
        sprite.color.set_alpha((1.0 - t) * 0.95);
        transform.rotation = Quat::from_rotation_z(t * 2.0);
        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
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
