use bevy::prelude::*;

use crate::arena::{ACTOR_SORT_OFFSET, Arena, Z_BRIGHT, random_edge_point, y_sort};
use crate::assets::GameAssets;
use crate::collision::{
    Collider, StaticCollider, move_circle_with_collisions, segment_aabb_hit, segment_circle_hit,
};
use crate::combat::{Bullet, Damage, Health, HitFlash, Lifetime, Velocity};
use crate::fx::{FadeOut, ScreenShake, spawn_burst, spawn_tracer};
use crate::healthbar::spawn_health_bar;
use crate::player::{Player, WalkAnim};
use crate::states::{GameSet, GameState, gameplay_active};
use vleue_navigator::prelude::{ManagedNavMesh, NavMesh, NavMeshStatus};

use std::collections::HashMap;

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
    /// Distance from the enemy origin to its barrel tip, in world units.
    pub muzzle_offset: f32,
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
    pub enabled: bool,
    pub timer: f32,
    pub interval: f32,
    pub min_interval: f32,
    pub wave_size: usize,
    pub max_alive: usize,
    pub elapsed: f32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            enabled: true,
            timer: 1.5,
            interval: 2.6,
            min_interval: 0.75,
            wave_size: 20,
            max_alive: 30,
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
                    report_navmesh_status,
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

fn report_navmesh_status(
    mut previous_statuses: Local<HashMap<Entity, NavMeshStatus>>,
    statuses: Query<(Entity, &NavMeshStatus), Changed<NavMeshStatus>>,
) {
    for (entity, status) in &statuses {
        let previous = previous_statuses.insert(entity, *status);
        match (previous, *status) {
            (_, NavMeshStatus::Built) => info!("Enemy navigation mesh is ready"),
            (_, NavMeshStatus::Failed) => {
                warn!("Enemy navigation mesh update failed; using fallback steering")
            }
            (Some(NavMeshStatus::Building), NavMeshStatus::Invalid) => {
                warn!("Enemy navigation mesh is invalid; using fallback steering")
            }
            (_, NavMeshStatus::Cancelled) => {
                debug!("Enemy navigation mesh update was cancelled")
            }
            _ => {}
        }
    }
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
            (64.0, 10.0, 22.0, 210.0, 1.15, true, 520.0),
            9.0,
        ),
        EnemyKind::Infected => (
            Vec2::new(24.0, 36.0),
            34.0,
            (96.0, 12.0, 0.0, 26.0, 0.7, false, 0.0),
            9.0,
        ),
    };

    let (speed, damage, muzzle_offset, attack_range, attack_interval, ranged, bullet_speed) = stats;

    let entity = commands
        .spawn((
            Sprite {
                image: frame_of(assets, kind, 0, 0),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(
                position.x,
                position.y,
                y_sort(position.y - ACTOR_SORT_OFFSET),
            ),
            Enemy {
                kind,
                speed,
                damage,
                muzzle_offset,
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
            EnemyPath::with_initial_jitter(),
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

    spawn_health_bar(commands, entity, position, 26.0, 25.0, true);
    entity
}

#[derive(Component)]
pub struct EnemyGun;

#[derive(Component, Default)]
struct EnemyPath {
    waypoints: Vec<Vec2>,
    next_waypoint: usize,
    target: Vec2,
    retry_timer: f32,
}

const PATH_REPATH_DISTANCE: f32 = 48.0;
const PATH_RETRY_INTERVAL: f32 = 0.4;
const WAYPOINT_REACHED_DISTANCE: f32 = 6.0;

impl EnemyPath {
    fn with_initial_jitter() -> Self {
        Self {
            retry_timer: fastrand::f32() * PATH_RETRY_INTERVAL,
            ..default()
        }
    }
}

fn refresh_enemy_path(
    path: &mut EnemyPath,
    navmesh: Option<&NavMesh>,
    start: Vec2,
    target: Vec2,
    dt: f32,
) {
    path.retry_timer = (path.retry_timer - dt).max(0.0);
    let route_exhausted = path.next_waypoint >= path.waypoints.len();
    let target_moved = path.target.distance_squared(target) >= PATH_REPATH_DISTANCE.powi(2);
    if path.retry_timer > 0.0 || (!route_exhausted && !target_moved) {
        return;
    }

    path.retry_timer = PATH_RETRY_INTERVAL;
    path.target = target;
    path.waypoints.clear();
    path.next_waypoint = 0;

    let Some(navmesh) = navmesh else {
        return;
    };
    let Some(route) = navmesh.transformed_path(start.extend(0.0), target.extend(0.0)) else {
        return;
    };

    path.waypoints = route
        .path
        .into_iter()
        .map(|point| point.truncate())
        .collect();
    if path.waypoints.len() > 1 {
        path.next_waypoint = 1;
    }
}

fn path_direction(path: &mut EnemyPath, position: Vec2) -> Option<Vec2> {
    while let Some(waypoint) = path.waypoints.get(path.next_waypoint).copied() {
        let delta = waypoint - position;
        if delta.length_squared() <= WAYPOINT_REACHED_DISTANCE.powi(2) {
            path.next_waypoint += 1;
        } else {
            return Some(delta.normalize_or_zero());
        }
    }
    None
}

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
    if !spawner.enabled {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.xy();
    let alive_count = alive.iter().count();

    spawner.elapsed += time.delta_secs();
    spawner.timer -= time.delta_secs();

    // ramp: interval shrinks toward the floor as the run goes on
    let ramp = (spawner.elapsed / 90.0).clamp(0.0, 1.0);
    let interval = spawner.interval + (spawner.min_interval - spawner.interval) * ramp;

    if spawner.timer > 0.0 {
        return;
    }
    let batch_size = spawn_batch_size(spawner.wave_size, alive_count, spawner.max_alive);
    if batch_size == 0 {
        return;
    }
    spawner.timer = interval;

    for _ in 0..batch_size {
        // Spawn on an edge, but never right on top of the player.
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
    }
}

fn spawn_batch_size(wave_size: usize, alive_count: usize, max_alive: usize) -> usize {
    wave_size.min(max_alive.saturating_sub(alive_count))
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
    navmeshes: Res<Assets<NavMesh>>,
    navmesh_entities: Query<(&ManagedNavMesh, &NavMeshStatus)>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Enemy,
            &mut WalkAnim,
            &Collider,
            &mut EnemyPath,
        ),
        With<Enemy>,
    >,
    statics: Query<(&Transform, &StaticCollider), Without<Enemy>>,
    mut static_colliders: Local<Vec<(Vec2, Collider)>>,
    mut damage: MessageWriter<Damage>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
    mut shake: ResMut<ScreenShake>,
) {
    let Ok((player_entity, player_transform, player_collider)) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.xy();
    let dt = time.delta_secs();
    let navmesh = navmesh_entities.iter().find_map(|(managed, status)| {
        (*status == NavMeshStatus::Built)
            .then(|| navmeshes.get(managed))
            .flatten()
    });
    let list = &snapshot.list;
    static_colliders.clear();
    static_colliders.extend(
        statics
            .iter()
            .map(|(static_transform, collider)| (static_transform.translation.xy(), collider.0)),
    );

    for (entity, mut transform, mut enemy, mut walk, collider, mut path) in &mut enemies {
        let position = transform.translation.xy();
        let radius = collider.radius();

        let to_player = player_position - position;
        let center_distance = to_player.length();
        let direction = to_player.normalize_or_zero();
        let attack_distance = enemy.attack_range + player_collider.radius() + radius;
        let can_see = !line_blocked(position, player_position, &statics);

        let path_direction = if center_distance > attack_distance * 0.9 {
            refresh_enemy_path(&mut path, navmesh, position, player_position, dt);
            path_direction(&mut path, position)
        } else {
            None
        };

        // desired movement: close to attack range, else hold / strafe
        let mut desired = if center_distance > attack_distance * 0.9 {
            path_direction.unwrap_or(direction) * enemy.speed
        } else if enemy.ranged && center_distance < attack_distance * 0.6 {
            -direction * enemy.speed * 0.55
        } else {
            Vec2::ZERO
        };

        // separation from other enemies
        for &(other_entity, other_position, other_radius) in list {
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

        // Keep the old steering fallback while the navmesh is building or if a path is unavailable.
        let mut steered = desired;
        if path_direction.is_none()
            && desired.length_squared() > 1.0
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

        let mut next_position =
            move_circle_with_collisions(position, steered * dt, radius, &static_colliders);

        for &(other_entity, other_position, other_radius) in list {
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
        transform.translation.z = y_sort(next_position.y - ACTOR_SORT_OFFSET);

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
                &mut images,
                &assets,
                position,
                face_angle,
                enemy.damage,
                enemy.bullet_speed,
                enemy.muzzle_offset,
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
    images: &mut Assets<Image>,
    assets: &GameAssets,
    position: Vec2,
    angle: f32,
    damage: f32,
    speed: f32,
    muzzle_offset: f32,
    shake: &mut ScreenShake,
) {
    let spread = (fastrand::f32() - 0.5) * 0.14;
    let shot_angle = angle + spread;
    let direction = Vec2::from_angle(shot_angle);
    let muzzle = position + direction * muzzle_offset;

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
        images,
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
        transform.translation.z = y_sort(transform.translation.y - ACTOR_SORT_OFFSET);
    }

    for (mut gun, parent) in &mut guns {
        let Ok(enemy) = parents.get(parent.parent()) else {
            continue;
        };
        let _ = enemy;
        gun.translation.z = 0.2;
    }
}

#[cfg(test)]
mod tests {
    use super::{EnemyPath, EnemySpawner, NavMesh, path_direction, spawn_batch_size};
    use bevy::prelude::{Vec2, default};

    #[test]
    fn enemies_are_enabled_by_default() {
        let spawner = EnemySpawner::default();
        assert!(spawner.enabled);
        assert_eq!(spawner.wave_size, 100);
        assert_eq!(spawner.max_alive, 100);
    }

    #[test]
    fn spawn_batch_fills_but_never_exceeds_the_alive_cap() {
        assert_eq!(spawn_batch_size(100, 0, 100), 100);
        assert_eq!(spawn_batch_size(100, 95, 100), 5);
        assert_eq!(spawn_batch_size(100, 100, 100), 0);
    }

    #[test]
    fn navmesh_routes_around_obstacles() {
        let navmesh = NavMesh::from_edge_and_obstacles(
            vec![
                Vec2::new(-100.0, -100.0),
                Vec2::new(100.0, -100.0),
                Vec2::new(100.0, 100.0),
                Vec2::new(-100.0, 100.0),
            ],
            vec![vec![
                Vec2::new(-10.0, -40.0),
                Vec2::new(10.0, -40.0),
                Vec2::new(10.0, 40.0),
                Vec2::new(-10.0, 40.0),
            ]],
        );

        let route = navmesh
            .transformed_path(
                Vec2::new(-80.0, 0.0).extend(0.0),
                Vec2::new(80.0, 0.0).extend(0.0),
            )
            .expect("a path should exist around the obstacle");

        assert!(
            route
                .path
                .iter()
                .any(|point| point.y.abs() >= 40.0 - f32::EPSILON)
        );
    }

    #[test]
    fn path_direction_advances_to_the_next_waypoint() {
        let mut path = EnemyPath {
            waypoints: vec![Vec2::ZERO, Vec2::new(20.0, 0.0), Vec2::new(20.0, 20.0)],
            next_waypoint: 1,
            ..default()
        };

        assert_eq!(
            path_direction(&mut path, Vec2::new(20.0, 0.0)),
            Some(Vec2::Y)
        );
        assert_eq!(path.next_waypoint, 2);
    }
}
