use bevy::prelude::*;

use crate::arena::{Arena, Z_BRIGHT, Z_LIGHT, random_edge_point, y_sort};
use crate::assets::GameAssets;
use crate::collision::{
    Collider, StaticCollider, circle_aabb_pushout, segment_aabb_hit, segment_circle_hit,
};
use crate::combat::{Bullet, Damage, Health, HitFlash, Lifetime, Velocity};
use crate::fx::{FadeOut, ScreenShake, spawn_burst, spawn_tracer};
use crate::healthbar::spawn_health_bar;
use crate::player::{Player, WalkAnim};
use crate::states::{GameSet, GameState, gameplay_active};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    /// Ranged bandit: keeps its distance and shoots.
    Raider,
    /// Fast melee mob: rushes and claws.
    Infected,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_interval: f32,
    pub attack_cooldown: f32,
    pub ranged: bool,
    pub bullet_speed: f32,
}

#[derive(Resource, Default)]
pub struct EnemySnapshot {
    pub list: Vec<(Entity, Vec2, f32)>,
}

/// Waves of edge-spawning raids that tighten over time.
#[derive(Resource)]
pub struct EnemySpawner {
    pub timer: f32,
    pub interval: f32,
    pub min_interval: f32,
    pub max_alive: usize,
    pub elapsed: f32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            timer: 1.5,
            interval: 2.6,
            min_interval: 0.75,
            max_alive: 18,
            elapsed: 0.0,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemySnapshot>()
            .init_resource::<EnemySpawner>()
            .add_systems(OnEnter(GameState::Playing), reset_spawner)
            .add_systems(
                Update,
                (
                    spawn_edge_enemies,
                    snapshot_enemies,
                    enemy_ai,
                    animate_enemies,
                )
                    .chain()
                    .in_set(GameSet::Ai)
                    .run_if(gameplay_active),
            );
    }
}

fn reset_spawner(mut spawner: ResMut<EnemySpawner>) {
    *spawner = EnemySpawner::default();
}

pub fn spawn_enemy(
    commands: &mut Commands,
    assets: &GameAssets,
    kind: EnemyKind,
    position: Vec2,
) -> Entity {
    let (size, base_hp, stats, radius) = match kind {
        EnemyKind::Raider => (
            Vec2::new(24.0, 36.0),
            55.0,
            (64.0, 10.0, 210.0, 1.15, true, 520.0),
            9.0,
        ),
        EnemyKind::Infected => (
            Vec2::new(24.0, 36.0),
            34.0,
            (96.0, 12.0, 26.0, 0.7, false, 0.0),
            9.0,
        ),
    };

    let (speed, damage, attack_range, attack_interval, ranged, bullet_speed) = stats;

    let entity = commands
        .spawn((
            Sprite {
                image: frame_of(assets, kind, 0, 0),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, y_sort(position.y)),
            Enemy {
                kind,
                speed,
                damage,
                attack_range,
                attack_interval,
                attack_cooldown: fastrand::f32() * attack_interval,
                ranged,
                bullet_speed,
            },
            Health::new(base_hp),
            Collider::circle(radius),
            HitFlash::default(),
            WalkAnim::default(),
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: assets.shadow_small.clone(),
                    custom_size: Some(Vec2::new(20.0, 13.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                    ..default()
                },
                Transform::from_xyz(2.0, -5.0, -1.4),
            ));
            if kind == EnemyKind::Raider {
                parent.spawn((
                    Sprite {
                        image: assets.raider_gun.clone(),
                        ..default()
                    },
                    Transform::from_xyz(7.0, 0.0, 0.2),
                    EnemyGun,
                ));
            }
        })
        .id();

    spawn_health_bar(commands, entity, 26.0, 25.0, true);
    entity
}

#[derive(Component)]
pub struct EnemyGun;

fn frame_of(assets: &GameAssets, kind: EnemyKind, dir: usize, frame: usize) -> Handle<Image> {
    match kind {
        EnemyKind::Raider => assets.raider_body[dir][frame].clone(),
        EnemyKind::Infected => assets.infected_body[dir][frame].clone(),
    }
}

/// 0=down 1=left 2=up 3=right, from a world-space aim angle.
fn dir_from_angle(angle: f32) -> usize {
    if angle.abs() < std::f32::consts::FRAC_PI_4 {
        3
    } else if angle > std::f32::consts::FRAC_PI_4 && angle < 3.0 * std::f32::consts::FRAC_PI_4 {
        2
    } else if angle < -std::f32::consts::FRAC_PI_4 && angle > -3.0 * std::f32::consts::FRAC_PI_4 {
        0
    } else {
        1
    }
}

fn spawn_edge_enemies(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    arena: Res<Arena>,
    mut spawner: ResMut<EnemySpawner>,
    alive: Query<(), With<Enemy>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.xy();

    spawner.elapsed += time.delta_secs();
    spawner.timer -= time.delta_secs();

    // ramp: interval shrinks toward the floor as the run goes on
    let ramp = (spawner.elapsed / 90.0).clamp(0.0, 1.0);
    let interval = spawner.interval + (spawner.min_interval - spawner.interval) * ramp;

    if spawner.timer > 0.0 || alive.iter().count() >= spawner.max_alive {
        return;
    }
    spawner.timer = interval;

    // spawn on an edge, but never right on top of the player
    let mut position = random_edge_point(&arena, 40.0);
    for _ in 0..6 {
        if position.distance(player_position) > 340.0 {
            break;
        }
        position = random_edge_point(&arena, 40.0);
    }

    let kind = if fastrand::f32() < 0.45 + 0.2 * ramp {
        EnemyKind::Infected
    } else {
        EnemyKind::Raider
    };

    spawn_enemy(&mut commands, &assets, kind, position);
    if fastrand::f32() < 0.35 + 0.3 * ramp {
        let offset = Vec2::from_angle(fastrand::f32() * std::f32::consts::TAU) * 40.0;
        let buddy = match kind {
            EnemyKind::Raider => EnemyKind::Infected,
            EnemyKind::Infected => EnemyKind::Infected,
        };
        spawn_enemy(&mut commands, &assets, buddy, position + offset);
    }
}

fn snapshot_enemies(
    mut snapshot: ResMut<EnemySnapshot>,
    enemies: Query<(Entity, &Transform, &Collider), With<Enemy>>,
) {
    snapshot.list.clear();
    snapshot
        .list
        .extend(enemies.iter().map(|(entity, transform, collider)| {
            (entity, transform.translation.xy(), collider.radius())
        }));
}

fn line_blocked(
    from: Vec2,
    to: Vec2,
    statics: &Query<(&Transform, &StaticCollider), Without<Enemy>>,
) -> bool {
    for (static_transform, static_collider) in statics {
        let hit = match static_collider.0 {
            Collider::Circle { radius } => {
                segment_circle_hit(from, to, static_transform.translation.xy(), radius).is_some()
            }
            Collider::Aabb { half } => {
                segment_aabb_hit(from, to, static_transform.translation.xy(), half)
            }
        };
        if hit {
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn enemy_ai(
    time: Res<Time>,
    arena: Res<Arena>,
    snapshot: Res<EnemySnapshot>,
    player: Query<(Entity, &Transform, &Collider), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy, &mut WalkAnim, &Collider), With<Enemy>>,
    statics: Query<(&Transform, &StaticCollider), Without<Enemy>>,
    mut damage: MessageWriter<Damage>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut shake: ResMut<ScreenShake>,
) {
    let Ok((player_entity, player_transform, player_collider)) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.xy();
    let dt = time.delta_secs();
    let list = snapshot.list.clone();

    for (entity, mut transform, mut enemy, mut walk, collider) in &mut enemies {
        let position = transform.translation.xy();
        let radius = collider.radius();

        let to_player = player_position - position;
        let center_distance = to_player.length();
        let direction = to_player.normalize_or_zero();
        let attack_distance = enemy.attack_range + player_collider.radius() + radius;
        let can_see = !line_blocked(position, player_position, &statics);

        // desired movement: close to attack range, else hold / strafe
        let mut desired = if center_distance > attack_distance * 0.9 {
            direction * enemy.speed
        } else if enemy.ranged && center_distance < attack_distance * 0.6 {
            -direction * enemy.speed * 0.55
        } else {
            Vec2::ZERO
        };

        // separation from other enemies
        for &(other_entity, other_position, other_radius) in &list {
            if other_entity == entity {
                continue;
            }
            let delta = position - other_position;
            let min_distance = radius + other_radius + 3.0;
            let distance_sq = delta.length_squared();
            if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                let distance = distance_sq.sqrt();
                desired += delta / distance * (enemy.speed * 0.9 * (1.0 - distance / min_distance));
            }
        }

        // slide along walls: if the direct path is blocked, try diagonal steers
        let mut steered = desired;
        if desired.length_squared() > 1.0
            && line_blocked(position, position + desired * 0.2, &statics)
        {
            let angle = desired.y.atan2(desired.x);
            let alt = [0.9, -0.9, 1.7, -1.7]
                .iter()
                .map(|&turn| Vec2::from_angle(angle + turn) * desired.length())
                .find(|candidate| !line_blocked(position, position + *candidate * 0.2, &statics));
            if let Some(candidate) = alt {
                steered = candidate;
            } else {
                steered = Vec2::ZERO;
            }
        }

        let mut next_position = position + steered * dt;

        for (static_transform, static_collider) in &statics {
            match static_collider.0 {
                Collider::Circle {
                    radius: static_radius,
                } => {
                    let delta = next_position - static_transform.translation.xy();
                    let min_distance = static_radius + radius;
                    let distance_sq = delta.length_squared();
                    if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
                        next_position += delta.normalize() * (min_distance - distance_sq.sqrt());
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
                next_position += delta / distance * (min_distance - distance);
            }
        }

        // never stand inside the survivor: shove out to touching distance,
        // which still leaves melee well within its reach
        let delta = next_position - player_position;
        let min_distance = radius + player_collider.radius();
        let distance_sq = delta.length_squared();
        if distance_sq < min_distance * min_distance && distance_sq > 0.0001 {
            let distance = distance_sq.sqrt();
            next_position += delta / distance * (min_distance - distance);
        }

        let limit = arena.half - Vec2::splat(radius + 6.0);
        next_position.x = next_position.x.clamp(-limit.x, limit.x);
        next_position.y = next_position.y.clamp(-limit.y, limit.y);

        transform.translation.x = next_position.x;
        transform.translation.y = next_position.y;
        transform.translation.z = y_sort(next_position.y);

        // facing: aim at the player when engaged, else face movement
        let face_angle = if can_see && center_distance < enemy.attack_range * 1.4 {
            to_player.y.atan2(to_player.x)
        } else if steered.length_squared() > 1.0 {
            steered.y.atan2(steered.x)
        } else {
            to_player.y.atan2(to_player.x)
        };
        walk.dir = dir_from_angle(face_angle);

        let moving = steered.length_squared() > 100.0;
        if moving {
            walk.timer += dt;
            let step = if enemy.kind == EnemyKind::Infected {
                0.09
            } else {
                0.12
            };
            if walk.timer >= step {
                walk.timer = 0.0;
                walk.frame = (walk.frame + 1) % 2;
            }
        } else {
            walk.timer = 0.0;
            walk.frame = 0;
        }

        // attack
        enemy.attack_cooldown -= dt;
        if enemy.attack_cooldown > 0.0 {
            continue;
        }

        if enemy.ranged {
            if !can_see || center_distance > attack_distance {
                continue;
            }
            enemy.attack_cooldown = enemy.attack_interval;
            fire_raider(
                &mut commands,
                &assets,
                position,
                face_angle,
                enemy.damage,
                enemy.bullet_speed,
                &mut shake,
            );
        } else {
            if center_distance > attack_distance + 4.0 {
                continue;
            }
            enemy.attack_cooldown = enemy.attack_interval;
            damage.write(Damage {
                target: player_entity,
                amount: enemy.damage,
            });
            shake.add(0.06);
            spawn_burst(
                &mut commands,
                &assets,
                player_position,
                Color::srgb(0.85, 0.2, 0.16),
                5,
                130.0,
                4.0,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fire_raider(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec2,
    angle: f32,
    damage: f32,
    speed: f32,
    shake: &mut ScreenShake,
) {
    let spread = (fastrand::f32() - 0.5) * 0.14;
    let shot_angle = angle + spread;
    let direction = Vec2::from_angle(shot_angle);
    let muzzle = position + direction * 15.0;

    commands.spawn((
        Sprite {
            image: assets.bullet.clone(),
            custom_size: Some(Vec2::new(11.0, 4.0)),
            color: Color::srgb(1.0, 0.66, 0.42),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT)
            .with_rotation(Quat::from_rotation_z(shot_angle)),
        Velocity(direction * speed),
        Bullet {
            damage,
            radius: 4.0,
            friendly: false,
        },
        Lifetime(2.0),
        DespawnOnExit(GameState::Playing),
    ));

    spawn_tracer(
        commands,
        assets,
        muzzle,
        muzzle + direction * 40.0,
        Color::srgba(1.0, 0.6, 0.35, 0.6),
        2.0,
        0.05,
    );

    commands.spawn((
        Sprite {
            image: assets.muzzle_flash.clone(),
            custom_size: Some(Vec2::splat(22.0)),
            color: Color::srgb(1.0, 0.78, 0.42),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT + 1.0)
            .with_rotation(Quat::from_rotation_z(shot_angle)),
        FadeOut {
            timer: 0.05,
            duration: 0.05,
        },
        DespawnOnExit(GameState::Playing),
    ));
    commands.spawn((
        Sprite {
            image: assets.glow_warm.clone(),
            custom_size: Some(Vec2::splat(130.0)),
            color: Color::srgba(1.0, 0.75, 0.42, 0.45),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_LIGHT),
        FadeOut {
            timer: 0.08,
            duration: 0.08,
        },
        DespawnOnExit(GameState::Playing),
    ));

    shake.add(0.03);
}

fn animate_enemies(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut enemies: Query<(&Enemy, &WalkAnim, &mut Sprite, &mut Transform), Changed<WalkAnim>>,
    mut guns: Query<(&mut Transform, &ChildOf), (With<EnemyGun>, Without<Enemy>)>,
    parents: Query<&Enemy>,
) {
    let _ = time;
    for (enemy, walk, mut sprite, mut transform) in &mut enemies {
        sprite.image = frame_of(&assets, enemy.kind, walk.dir, walk.frame);
        sprite.custom_size = Some(Vec2::new(24.0, 36.0));
        transform.translation.z = y_sort(transform.translation.y);
    }

    for (mut gun, parent) in &mut guns {
        let Ok(enemy) = parents.get(parent.parent()) else {
            continue;
        };
        let _ = enemy;
        gun.translation.z = 0.2;
    }
}
