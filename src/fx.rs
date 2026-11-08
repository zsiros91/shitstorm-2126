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
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>().add_systems(
            Update,
            (
                enemy_death_fx,
                animate_explosions,
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
    position: Vec2,
    color: Color,
    count: usize,
    speed: f32,
    size: f32,
) {
    for _ in 0..count {
        let dir = Vec2::from_angle(fastrand::f32() * std::f32::consts::TAU);
        let spd = speed * (0.25 + fastrand::f32() * 0.75);
        let life = 0.18 + fastrand::f32() * 0.28;
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(position.x, position.y, 20.0),
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

pub fn spawn_impact(commands: &mut Commands, position: Vec2) {
    spawn_burst(
        commands,
        position,
        Color::srgb(1.0, 0.85, 0.35),
        5,
        150.0,
        3.0,
    );
}

pub fn spawn_blood(commands: &mut Commands, position: Vec2) {
    spawn_burst(
        commands,
        position,
        Color::srgb(0.65, 0.08, 0.06),
        10,
        120.0,
        4.0,
    );
    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.32, 0.04, 0.03, 0.65),
            Vec2::splat(7.0),
        ),
        Transform::from_xyz(position.x, position.y, -8.0),
        FadeOut {
            timer: 6.0,
            duration: 6.0,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

pub fn spawn_explosion(commands: &mut Commands, assets: &GameAssets, position: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.explosions[0].clone(),
            custom_size: Some(Vec2::splat(76.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 18.0),
        ExplosionAnim {
            frames: assets.explosions.to_vec(),
            index: 0,
            timer: 0.0,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

fn enemy_death_fx(
    mut kills: MessageReader<EnemyKilled>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
) {
    for kill in kills.read() {
        spawn_explosion(&mut commands, &assets, kill.position);
        spawn_blood(&mut commands, kill.position);
        shake.add(0.14);
    }
}

fn animate_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut ExplosionAnim, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut explosions {
        anim.timer += time.delta_secs();
        while anim.timer >= 0.055 {
            anim.timer -= 0.055;
            anim.index += 1;
            if anim.index >= anim.frames.len() {
                commands.entity(entity).despawn();
                break;
            }
            sprite.image = anim.frames[anim.index].clone();
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
        if fade.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            sprite.color.set_alpha((fade.timer / fade.duration).clamp(0.0, 1.0));
        }
    }
}
