use bevy::prelude::*;

use crate::arena::Arena;
use crate::assets::GameAssets;
use crate::collision::{circle_aabb_pushout, Collider, StaticCollider};
use crate::combat::{Damage, Health, HitFlash};
use crate::fx::ScreenShake;
use crate::hq::Hq;
use crate::healthbar::spawn_health_bar;
use crate::player::Player;
use crate::states::{gameplay_active, GameSet, GameState};
use crate::tower::Tower;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Zergling,
    Hydralisk,
    Ultralisk,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_interval: f32,
    pub attack_cooldown: f32,
    pub score: u32,
    pub reward_minerals: u32,
    pub reward_gas: u32,
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
) -> Entity {
    let wave_index = wave.saturating_sub(1) as f32;
    let health_scale = 1.0 + 0.13 * wave_index;
    let damage_scale = 1.0 + 0.09 * wave_index;

    let (image, size, base_hp, stats, radius, shadow_size) = match kind {
        EnemyKind::Zergling => (
            assets.enemy_zergling.clone(),
            Vec2::splat(50.0),
            30.0,
            (162.0, 7.0, 38.0, 0.55, 10, 6, 0),
            15.0,
            48.0,
        ),
        EnemyKind::Hydralisk => (
            assets.enemy_hydralisk.clone(),
            Vec2::splat(64.0),
            62.0,
            (88.0, 14.0, 54.0, 1.0, 18, 10, 0),
            19.0,
            62.0,
        ),
        EnemyKind::Ultralisk => (
            assets.enemy_ultralisk.clone(),
            Vec2::splat(98.0),
            240.0,
            (60.0, 34.0, 66.0, 1.4, 50, 28, 12),
            30.0,
            100.0,
        ),
    };

    let (speed, damage, attack_range, attack_interval, score, minerals, gas) = stats;
    let entity = commands
        .spawn((
            Sprite {
                image,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 9.0),
            Enemy {
                kind,
                speed,
                damage: damage * damage_scale,
                attack_range,
                attack_interval,
                attack_cooldown: fastrand::f32() * attack_interval,
                score,
                reward_minerals: minerals,
                reward_gas: gas,
            },
            Health::new(base_hp * health_scale),
            Collider::circle(radius),
            HitFlash::default(),
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: assets.shadow.clone(),
                    custom_size: Some(Vec2::splat(shadow_size)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                    ..default()
                },
                Transform::from_xyz(4.0, -6.0, -7.0),
            ));
        })
        .id();

    spawn_health_bar(commands, entity, 36.0, 32.0, true);
    entity
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
    hq: Query<(Entity, &Transform, &Collider), (With<Hq>, Without<Enemy>)>,
    player: Query<(Entity, &Transform, &Collider), (With<Player>, Without<Enemy>)>,
    towers: Query<(Entity, &Transform, &Collider), (With<Tower>, Without<Enemy>)>,
    mut enemies: Query<
        (Entity, &mut Transform, &mut Enemy, &Collider),
        (With<Enemy>, Without<Player>),
    >,
    statics: Query<(&Transform, &StaticCollider), Without<Enemy>>,
    mut damage: MessageWriter<Damage>,
    mut shake: ResMut<ScreenShake>,
) {
    let Ok((hq_entity, hq_transform, hq_collider)) = hq.single() else {
        return;
    };
    let hq_position = hq_transform.translation.xy();
    let player_target = player.single().ok();
    let dt = time.delta_secs();
    let list = snapshot.list.clone();

    for (entity, mut transform, mut enemy, collider) in &mut enemies {
        let position = transform.translation.xy();
        let radius = collider.radius();

        let mut target = (hq_entity, hq_position, hq_collider.radius());
        let mut best_distance = position.distance(hq_position) - hq_collider.radius() + 120.0;

        if let Some((player_entity, player_transform, player_collider)) = player_target {
            let player_position = player_transform.translation.xy();
            let distance = position.distance(player_position) - player_collider.radius();
            if position.distance(player_position) < 250.0 && distance < best_distance {
                target = (player_entity, player_position, player_collider.radius());
                best_distance = distance;
            }
        }

        for (tower_entity, tower_transform, tower_collider) in &towers {
            let tower_position = tower_transform.translation.xy();
            let distance = position.distance(tower_position) - tower_collider.radius();
            if position.distance(tower_position) < 260.0 && distance < best_distance {
                target = (tower_entity, tower_position, tower_collider.radius());
                best_distance = distance;
            }
        }

        let to_target = target.1 - position;
        let center_distance = to_target.length();
        let attack_distance = enemy.attack_range + target.2 + radius;
        let direction = to_target.normalize_or_zero();

        let mut desired = if center_distance > attack_distance * 0.88 {
            direction * enemy.speed
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
                    * (enemy.speed * 0.9 * (1.0 - distance / min_distance));
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

        let limit = arena.half - Vec2::splat(radius + 6.0);
        next_position.x = next_position.x.clamp(-limit.x, limit.x);
        next_position.y = next_position.y.clamp(-limit.y, limit.y);

        transform.translation.x = next_position.x;
        transform.translation.y = next_position.y;

        if desired.length_squared() > 100.0 {
            let target_angle = desired.y.atan2(desired.x);
            transform.rotation = transform
                .rotation
                .slerp(Quat::from_rotation_z(target_angle), (10.0 * dt).min(1.0));
        }

        enemy.attack_cooldown -= dt;
        if center_distance <= attack_distance + 6.0 && enemy.attack_cooldown <= 0.0 {
            enemy.attack_cooldown = enemy.attack_interval;
            damage.write(Damage {
                target: target.0,
                amount: enemy.damage,
            });
            if target.0 == hq_entity
                || player_target.map(|(e, _, _)| e) == Some(target.0)
            {
                shake.add(0.05);
            }
        }
    }
}
