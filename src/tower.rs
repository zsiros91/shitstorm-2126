use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::collision::{Collider, StaticCollider};
use crate::combat::{Bullet, BulletKind, Damage, Health, HitFlash, Lifetime, Velocity};
use crate::enemy::EnemySnapshot;
use crate::fx::{spawn_burst, spawn_lightning, spawn_tracer, ScreenShake};
use crate::healthbar::spawn_health_bar;
use crate::power::{PowerNode, RELAY_LINK_RANGE, TOWER_LINK_RANGE};
use crate::states::GameState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerKind {
    Gauss,
    Missile,
    Arc,
    Relay,
}

impl TowerKind {
    pub const ALL: [TowerKind; 4] = [
        TowerKind::Gauss,
        TowerKind::Missile,
        TowerKind::Arc,
        TowerKind::Relay,
    ];

    pub fn title(self) -> &'static str {
        match self {
            TowerKind::Gauss => "GAUSS TURRET",
            TowerKind::Missile => "MISSILE BATTERY",
            TowerKind::Arc => "ARC PROJECTOR",
            TowerKind::Relay => "POWER RELAY",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            TowerKind::Gauss => "GAUSS",
            TowerKind::Missile => "MISSILE",
            TowerKind::Arc => "ARC",
            TowerKind::Relay => "RELAY",
        }
    }

    pub fn cost(self) -> (u32, u32) {
        match self {
            TowerKind::Gauss => (50, 0),
            TowerKind::Missile => (75, 25),
            TowerKind::Arc => (60, 20),
            TowerKind::Relay => (25, 0),
        }
    }

    pub fn hotkey(self) -> &'static str {
        match self {
            TowerKind::Gauss => "1",
            TowerKind::Missile => "2",
            TowerKind::Arc => "3",
            TowerKind::Relay => "4",
        }
    }

    pub fn range(self) -> f32 {
        match self {
            TowerKind::Gauss => 300.0,
            TowerKind::Missile => 380.0,
            TowerKind::Arc => 250.0,
            TowerKind::Relay => 0.0,
        }
    }

    pub fn link_range(self) -> f32 {
        match self {
            TowerKind::Relay => RELAY_LINK_RANGE,
            _ => TOWER_LINK_RANGE,
        }
    }

    pub fn max_health(self) -> f32 {
        match self {
            TowerKind::Gauss => 260.0,
            TowerKind::Missile => 320.0,
            TowerKind::Arc => 280.0,
            TowerKind::Relay => 180.0,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            TowerKind::Gauss => "Rapid kinetic rounds. Cheap and reliable.",
            TowerKind::Missile => "Slow splash missiles. Breaks up swarms.",
            TowerKind::Arc => "Short-range chain lightning.",
            TowerKind::Relay => "Extends the command power chain.",
        }
    }

    pub fn icon(self, assets: &GameAssets) -> Handle<Image> {
        match self {
            TowerKind::Gauss => assets.icon_gauss.clone(),
            TowerKind::Missile => assets.icon_missile.clone(),
            TowerKind::Arc => assets.icon_arc.clone(),
            TowerKind::Relay => assets.icon_relay.clone(),
        }
    }

    pub fn sprite(self, assets: &GameAssets) -> Handle<Image> {
        match self {
            TowerKind::Gauss => assets.gauss_base.clone(),
            TowerKind::Missile => assets.missile_base.clone(),
            TowerKind::Arc => assets.arc_base.clone(),
            TowerKind::Relay => assets.relay.clone(),
        }
    }

    pub fn head_sprite(self, assets: &GameAssets) -> Option<Handle<Image>> {
        match self {
            TowerKind::Gauss => Some(assets.gauss_head.clone()),
            TowerKind::Missile => Some(assets.missile_head.clone()),
            TowerKind::Arc => Some(assets.arc_head.clone()),
            TowerKind::Relay => None,
        }
    }

    pub fn size(self) -> f32 {
        match self {
            TowerKind::Relay => 62.0,
            _ => 66.0,
        }
    }
}

#[derive(Component)]
pub struct Tower {
    pub kind: TowerKind,
    pub range: f32,
    pub damage: f32,
    pub fire_interval: f32,
    pub cooldown: f32,
    pub splash: f32,
    pub slow: f32,
}

#[derive(Component)]
pub struct TowerHead;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (tower_ai, sync_tower_power).in_set(crate::states::GameSet::Combat),
        );
    }
}

pub fn tower_stats(kind: TowerKind) -> (f32, f32, f32, f32) {
    match kind {
        TowerKind::Gauss => (12.0, 0.22, 0.0, 0.0),
        TowerKind::Missile => (42.0, 1.25, 92.0, 0.0),
        TowerKind::Arc => (24.0, 1.05, 0.0, 0.35),
        TowerKind::Relay => (0.0, 1.0, 0.0, 0.0),
    }
}

pub fn spawn_tower(
    commands: &mut Commands,
    assets: &GameAssets,
    kind: TowerKind,
    position: Vec2,
) -> Entity {
    let size = kind.size();
    let (damage, fire_interval, splash, slow) = tower_stats(kind);
    let entity = commands
        .spawn((
            Sprite {
                image: kind.sprite(assets),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 6.0),
            Tower {
                kind,
                range: kind.range(),
                damage,
                fire_interval,
                cooldown: fastrand::f32() * fire_interval,
                splash,
                slow,
            },
            Health::new(kind.max_health()),
            HitFlash::default(),
            Collider::circle(28.0),
            StaticCollider(Collider::circle(30.0)),
            PowerNode::tower(kind.link_range()),
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            if let Some(head) = kind.head_sprite(assets) {
                parent.spawn((
                    Sprite {
                        image: head,
                        custom_size: Some(Vec2::splat(58.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    TowerHead,
                ));
            }
            // Power status glow at the base.
            parent.spawn((
                Sprite {
                    image: assets.ring.clone(),
                    custom_size: Some(Vec2::splat(48.0)),
                    color: Color::srgba(0.25, 0.9, 1.0, 0.16),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.5),
            ));
            parent.spawn((
                Sprite {
                    image: assets.shadow.clone(),
                    custom_size: Some(Vec2::splat(size + 18.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.55),
                    ..default()
                },
                Transform::from_xyz(5.0, -7.0, -5.0),
            ));
        })
        .id();

    spawn_health_bar(commands, entity, 48.0, 45.0, false);
    entity
}

fn tower_ai(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    snapshot: Res<EnemySnapshot>,
    mut towers: Query<(&Transform, &mut Tower, &PowerNode, &Children)>,
    mut heads: Query<&mut Transform, (With<TowerHead>, Without<Tower>)>,
    mut damage: MessageWriter<Damage>,
    mut shake: ResMut<ScreenShake>,
) {
    let dt = time.delta_secs();

    for (transform, mut tower, power, children) in &mut towers {
        tower.cooldown = (tower.cooldown - dt).max(0.0);
        if !power.powered || tower.kind == TowerKind::Relay {
            continue;
        }

        let position = transform.translation.xy();
        let target = snapshot
            .list
            .iter()
            .filter(|(_, enemy_pos, _)| enemy_pos.distance(position) <= tower.range)
            .min_by(|a, b| {
                a.1.distance_squared(position)
                    .total_cmp(&b.1.distance_squared(position))
            });

        let Some((target_entity, target_position, _)) = target else {
            continue;
        };

        let direction = (*target_position - position).normalize_or_zero();
        let angle = direction.y.atan2(direction.x);

        if let Some(head_entity) = children.first() {
            if let Ok(mut head_transform) = heads.get_mut(*head_entity) {
                head_transform.rotation =
                    Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
            }
        }

        if tower.cooldown > 0.0 {
            continue;
        }
        tower.cooldown = tower.fire_interval;

        match tower.kind {
            TowerKind::Gauss => {
                let muzzle = position + direction * 30.0;
                spawn_bullet(
                    &mut commands,
                    &assets,
                    BulletKind::Gauss,
                    muzzle,
                    direction,
                    tower.damage,
                    tower.splash,
                );
                spawn_tracer(
                    &mut commands,
                    &assets,
                    muzzle,
                    muzzle + direction * 80.0,
                    Color::srgba(0.45, 0.95, 1.0, 0.85),
                    2.0,
                    0.06,
                );
                spawn_muzzle(&mut commands, &assets, muzzle, angle);
            }
            TowerKind::Missile => {
                let muzzle = position + direction * 32.0;
                spawn_bullet(
                    &mut commands,
                    &assets,
                    BulletKind::Missile,
                    muzzle,
                    direction,
                    tower.damage,
                    tower.splash,
                );
                spawn_muzzle(&mut commands, &assets, muzzle, angle);
                spawn_burst(
                    &mut commands,
                    &assets,
                    muzzle,
                    Color::srgb(1.0, 0.7, 0.25),
                    5,
                    90.0,
                    3.0,
                );
            }
            TowerKind::Arc => {
                let mut hit = vec![*target_entity];
                damage.write(Damage {
                    target: *target_entity,
                    amount: tower.damage,
                });
                spawn_lightning(
                    &mut commands,
                    &assets,
                    position,
                    *target_position,
                    Color::srgba(0.45, 0.95, 1.0, 0.95),
                );

                let mut previous = *target_position;
                for _ in 0..2 {
                    let next = snapshot
                        .list
                        .iter()
                        .filter(|(entity, pos, _)| {
                            !hit.contains(entity) && pos.distance(previous) <= 150.0
                        })
                        .min_by(|a, b| {
                            a.1.distance_squared(previous)
                                .total_cmp(&b.1.distance_squared(previous))
                        });
                    if let Some((entity, pos, _)) = next {
                        hit.push(*entity);
                        damage.write(Damage {
                            target: *entity,
                            amount: tower.damage * 0.65,
                        });
                        spawn_lightning(
                            &mut commands,
                            &assets,
                            previous,
                            *pos,
                            Color::srgba(0.55, 0.95, 1.0, 0.8),
                        );
                        previous = *pos;
                    } else {
                        break;
                    }
                }
                spawn_burst(
                    &mut commands,
                    &assets,
                    *target_position,
                    Color::srgb(0.5, 0.9, 1.0),
                    8,
                    120.0,
                    3.0,
                );
                shake.add(0.03);
            }
            TowerKind::Relay => {}
        }
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    assets: &GameAssets,
    kind: BulletKind,
    position: Vec2,
    direction: Vec2,
    damage: f32,
    splash: f32,
) {
    let (image, size, speed, lifetime, radius) = match kind {
        BulletKind::Gauss => (assets.bullet_gauss.clone(), Vec2::new(10.0, 26.0), 980.0, 1.2, 5.0),
        BulletKind::Missile => (assets.bullet_missile.clone(), Vec2::new(16.0, 34.0), 620.0, 2.2, 8.0),
    };
    let angle = direction.y.atan2(direction.x);
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 16.0)
            .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2)),
        Velocity(direction * speed),
        Bullet {
            damage,
            radius,
            splash,
            slow: 0.0,
            kind,
        },
        Lifetime(lifetime),
        DespawnOnExit(GameState::Playing),
    ));
}

fn spawn_muzzle(commands: &mut Commands, assets: &GameAssets, position: Vec2, angle: f32) {
    commands.spawn((
        Sprite {
            image: assets.muzzle_flash.clone(),
            custom_size: Some(Vec2::splat(28.0)),
            color: Color::srgb(1.0, 0.82, 0.4),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 24.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        crate::fx::FadeOut {
            timer: 0.06,
            duration: 0.06,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

fn sync_tower_power(mut towers: Query<(&PowerNode, &mut Sprite), With<Tower>>) {
    for (power, mut sprite) in &mut towers {
        sprite.color = if power.powered {
            Color::WHITE
        } else {
            Color::srgb(0.52, 0.42, 0.46)
        };
    }
}
