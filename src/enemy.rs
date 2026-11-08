use bevy::prelude::*;

use crate::arena::Arena;
use crate::assets::GameAssets;
use crate::collision::{circle_aabb_pushout, Collider, StaticCollider};
use crate::combat::{Damage, Health, HitFlash};
use crate::fx::ScreenShake;
use crate::player::Player;
use crate::states::{gameplay_active, GameSet, GameState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Walker,
    Runner,
    Brute,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Enemy {
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_interval: f32,
    pub attack_cooldown: f32,
    pub score: u32,
}

#[derive(Resource, Default)]
pub struct EnemySnapshot {
    pub list: Vec<(Entity, Vec2, f32)>,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemySnapshot>().add_systems(
            Update,
            (snapshot_enemies, enemy_ai)
                .chain()
                .in_set(GameSet::Ai)
                .run_if(gameplay_active),
        );
    }
}

pub fn spawn_enemy(
    commands: &mut Commands,
    assets: &GameAssets,
    kind: EnemyKind,
    position: Vec2,
    wave: u32,
) {
    let wave_index = wave.saturating_sub(1) as f32;
    let health_scale = 1.0 + 0.12 * wave_index;
    let damage_scale = 1.0 + 0.08 * wave_index;

    let (image, size, base_hp, stats) = match kind {
        EnemyKind::Walker => (
            assets.enemy_walker.clone(),
            Vec2::new(35.0, 43.0),
            35.0,
            (85.0, 9.0, 44.0, 0.9, 10),
        ),
        EnemyKind::Runner => (
            assets.enemy_runner.clone(),
            Vec2::new(49.0, 43.0),
            22.0,
            (150.0, 6.0, 40.0, 0.55, 15),
        ),
        EnemyKind::Brute => (
            assets.enemy_brute.clone(),
            Vec2::new(49.0, 43.0),
            130.0,
            (60.0, 22.0, 54.0, 1.4, 40),
        ),
    };

    let (speed, damage, attack_range, attack_interval, score) = stats;
    let radius = match kind {
        EnemyKind::Brute => 20.0,
        _ => 15.0,
    };

    commands.spawn((
        Sprite {
            image,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 9.0),
        Enemy {
            speed,
            damage: damage * damage_scale,
            attack_range,
            attack_interval,
            attack_cooldown: fastrand::f32() * attack_interval,
            score,
        },
        Health {
            current: base_hp * health_scale,
            max: base_hp * health_scale,
        },
        Collider::circle(radius),
        HitFlash::default(),
        DespawnOnExit(GameState::Playing),
    ));
}

fn snapshot_enemies(
    mut snapshot: ResMut<EnemySnapshot>,
    enemies: Query<(Entity, &Transform, &Collider), With<Enemy>>,
) {
    snapshot.list.clear();
    snapshot.list.extend(
        enemies
            .iter()
            .map(|(entity, transform, collider)| {
                (entity, transform.translation.xy(), collider.radius())
            }),
    );
}

fn enemy_ai(
    time: Res<Time>,
    arena: Res<Arena>,
    snapshot: Res<EnemySnapshot>,
    player: Query<(Entity, &Transform), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<
        (Entity, &mut Transform, &mut Enemy, &Collider),
        (With<Enemy>, Without<Player>),
    >,
    statics: Query<(&Transform, &StaticCollider), Without<Enemy>>,
    mut damage: MessageWriter<Damage>,
    mut shake: ResMut<ScreenShake>,
) {
    let Ok((player_entity, player_transform)) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.xy();
    let dt = time.delta_secs();
    let list = snapshot.list.clone();

    for (entity, mut transform, mut enemy, collider) in &mut enemies {
        let position = transform.translation.xy();
        let radius = collider.radius();
        let to_player = player_position - position;
        let distance = to_player.length();

        let mut desired = if distance > enemy.attack_range * 0.8 {
            to_player.normalize_or_zero() * enemy.speed
        } else {
            Vec2::ZERO
        };

        for &(other_entity, other_position, other_radius) in &list {
            if other_entity == entity {
                continue;
            }
            let delta = position - other_position;
            let min_distance = radius + other_radius + 3.0;
            let distance_sq = delta.length_squared();
            if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                let distance = distance_sq.sqrt();
                desired += delta / distance
                    * (enemy.speed * 0.85 * (1.0 - distance / min_distance));
            }
        }

        let mut next_position = position + desired * dt;

        for (static_transform, static_collider) in &statics {
            match static_collider.0 {
                Collider::Circle {
                    radius: static_radius,
                } => {
                    let delta = next_position - static_transform.translation.xy();
                    let min_distance = static_radius + radius;
                    let distance_sq = delta.length_squared();
                    if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                        next_position +=
                            delta.normalize() * (min_distance - distance_sq.sqrt());
                    }
                }
                Collider::Aabb { half } => {
                    if let Some(push) = circle_aabb_pushout(
                        next_position,
                        radius,
                        static_transform.translation.xy(),
                        half,
                    ) {
                        next_position += push;
                    }
                }
            }
        }

        for &(other_entity, other_position, other_radius) in &list {
            if other_entity == entity {
                continue;
            }
            let delta = next_position - other_position;
            let min_distance = radius + other_radius;
            let distance_sq = delta.length_squared();
            if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                let distance = distance_sq.sqrt();
                next_position += delta / distance * (min_distance - distance) * 0.5;
            }
        }

        let limit = arena.half - Vec2::splat(radius + 4.0);
        next_position.x = next_position.x.clamp(-limit.x, limit.x);
        next_position.y = next_position.y.clamp(-limit.y, limit.y);

        transform.translation.x = next_position.x;
        transform.translation.y = next_position.y;

        if desired.length_squared() > 100.0 {
            let target = desired.y.atan2(desired.x);
            transform.rotation = transform
                .rotation
                .slerp(Quat::from_rotation_z(target), (10.0 * dt).min(1.0));
        }

        enemy.attack_cooldown -= dt;
        if distance <= enemy.attack_range && enemy.attack_cooldown <= 0.0 {
            enemy.attack_cooldown = enemy.attack_interval;
            damage.write(Damage {
                target: player_entity,
                amount: enemy.damage,
            });
            shake.add(0.05);
        }
    }
}
