use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::arena::{NavigationObstacle, Z_BRIGHT, y_sort};
use crate::assets::GameAssets;
use crate::camera::OffscreenRenderCamera;
use crate::collision::{Collider, StaticCollider};
use crate::combat::{Bullet, Health, HitFlash, Lifetime, Velocity};
use crate::enemy::Enemy;
use crate::fx::{FadeOut, spawn_casing};
use crate::healthbar::spawn_health_bar;
use crate::player::{Aim, Player};
use crate::states::{GameSet, GameState, gameplay_active};
use vleue_navigator::prelude::PrimitiveObstacle;

const TURRET_SIZE: Vec2 = Vec2::splat(38.0);
const TURRET_HEAD_RENDER_SIZE: u32 = 40;
const TURRET_COLLISION_RADIUS: f32 = 17.0;
const TURRET_PLACEMENT_OFFSET: f32 = 40.0;
const TURRET_MAX_AMMO: u32 = 1_000;
const TURRET_HEALTH: f32 = 180.0;
const TURRET_RANGE: f32 = 420.0;
const TURRET_HEAD_TURN_SPEED: f32 = 1.5;
const TURRET_AIM_TOLERANCE: f32 = 0.08;
const TURRET_FIRE_INTERVAL: f32 = 0.18;
const TURRET_DAMAGE: f32 = 18.0;
const TURRET_BULLET_SPEED: f32 = 820.0;
const TURRET_MUZZLE_OFFSET: f32 = 23.0;

#[derive(Component)]
struct Turret {
    ammo: u32,
    max_ammo: u32,
    cooldown: f32,
    range: f32,
    turn_speed: f32,
    fire_interval: f32,
    damage: f32,
    bullet_speed: f32,
    muzzle_offset: f32,
}

impl Turret {
    fn basic() -> Self {
        Self {
            ammo: TURRET_MAX_AMMO,
            max_ammo: TURRET_MAX_AMMO,
            cooldown: 0.0,
            range: TURRET_RANGE,
            turn_speed: TURRET_HEAD_TURN_SPEED,
            fire_interval: TURRET_FIRE_INTERVAL,
            damage: TURRET_DAMAGE,
            bullet_speed: TURRET_BULLET_SPEED,
            muzzle_offset: TURRET_MUZZLE_OFFSET,
        }
    }
}

#[derive(Component)]
struct TurretHeadSource;

#[derive(Component)]
struct TurretHead;

#[derive(Component)]
struct TurretAmmoBar {
    owner: Entity,
    width: f32,
    offset_y: f32,
    fill: bool,
}

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            place_turret.in_set(GameSet::Input).run_if(gameplay_active),
        )
        .add_systems(
            Update,
            update_turrets
                .in_set(GameSet::Combat)
                .before(crate::combat::bullet_collision)
                .run_if(gameplay_active),
        )
        .add_systems(
            Update,
            update_turret_ammo_bars
                .in_set(GameSet::Ui)
                .run_if(gameplay_active),
        );
    }
}

fn place_turret(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    assets: Res<GameAssets>,
    player: Query<(&Transform, &Aim), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }

    let Ok((player_transform, aim)) = player.single() else {
        return;
    };
    spawn_turret(
        &mut commands,
        &mut images,
        &assets,
        turret_position_next_to_player(player_transform.translation.xy(), aim.angle),
    );
}

fn turret_position_next_to_player(player_position: Vec2, aim_angle: f32) -> Vec2 {
    player_position + Vec2::from_angle(aim_angle) * TURRET_PLACEMENT_OFFSET
}

fn spawn_turret(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    assets: &GameAssets,
    position: Vec2,
) -> Entity {
    let target = images.add(Image::new_target_texture(
        TURRET_HEAD_RENDER_SIZE,
        TURRET_HEAD_RENDER_SIZE,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    let entity = commands
        .spawn((
            Sprite {
                image: assets.turret.clone(),
                custom_size: Some(TURRET_SIZE),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, y_sort(position.y - 19.0)),
            Turret::basic(),
            Health::new(TURRET_HEALTH),
            HitFlash::default(),
            Collider::circle(TURRET_COLLISION_RADIUS),
            StaticCollider(Collider::circle(TURRET_COLLISION_RADIUS)),
            PrimitiveObstacle::Circle(Circle::new(TURRET_COLLISION_RADIUS)),
            NavigationObstacle,
            DespawnOnExit(GameState::Playing),
        ))
        .id();
    let render_layers = RenderLayers::from_layers(&[entity.index_u32() as usize + 2]);
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: target.clone(),
                custom_size: Some(Vec2::splat(TURRET_HEAD_RENDER_SIZE as f32)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
            TurretHead,
        ));
        parent.spawn((
            Sprite::from_image(assets.turret_head.clone()),
            Transform::IDENTITY,
            TurretHeadSource,
            render_layers.clone(),
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
            render_layers,
            OffscreenRenderCamera,
        ));
    });

    spawn_health_bar(commands, entity, position, 32.0, 30.0, false);
    spawn_turret_ammo_bar(commands, entity, position);
    entity
}

fn spawn_turret_ammo_bar(commands: &mut Commands, owner: Entity, position: Vec2) {
    const WIDTH: f32 = 32.0;
    const OFFSET_Y: f32 = 22.0;

    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.03, 0.04, 0.05, 0.85),
            Vec2::new(WIDTH + 4.0, 6.0),
        ),
        Transform::from_xyz(position.x, position.y + OFFSET_Y, Z_BRIGHT + 3.0),
        TurretAmmoBar {
            owner,
            width: WIDTH,
            offset_y: OFFSET_Y,
            fill: false,
        },
        DespawnOnExit(GameState::Playing),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.94, 0.68, 0.18), Vec2::new(WIDTH, 3.0)),
        Transform::from_xyz(position.x, position.y + OFFSET_Y, Z_BRIGHT + 4.0),
        TurretAmmoBar {
            owner,
            width: WIDTH,
            offset_y: OFFSET_Y,
            fill: true,
        },
        DespawnOnExit(GameState::Playing),
    ));
}

fn update_turrets(
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<TurretHeadSource>)>,
    mut turrets: Query<(Entity, &Transform, &mut Turret), (Without<TurretHead>, Without<Enemy>)>,
    mut heads: Query<(&ChildOf, &mut Transform), (With<TurretHeadSource>, Without<Turret>)>,
) {
    let dt = time.delta_secs();
    for (turret_entity, turret_transform, mut turret) in &mut turrets {
        turret.cooldown = (turret.cooldown - dt).max(0.0);
        let origin = turret_transform.translation.xy();
        let target = closest_target(
            origin,
            enemies
                .iter()
                .map(|(entity, transform)| (entity, transform.translation.xy())),
            turret.range,
        );
        let Some((_, target_position)) = target else {
            continue;
        };

        let delta = target_position - origin;
        let target_angle = delta.y.atan2(delta.x);
        let Some((_, mut head_transform)) = heads
            .iter_mut()
            .find(|(parent, _)| parent.parent() == turret_entity)
        else {
            continue;
        };
        let current_angle = head_transform.rotation.to_euler(EulerRot::XYZ).2;
        let angle = rotate_angle_towards(current_angle, target_angle, turret.turn_speed * dt);
        head_transform.rotation = Quat::from_rotation_z(angle);

        if turret.cooldown > 0.0
            || turret.ammo == 0
            || shortest_angle_difference(angle, target_angle).abs() > TURRET_AIM_TOLERANCE
        {
            continue;
        }

        turret.ammo -= 1;
        turret.cooldown = turret.fire_interval;
        fire_turret(&mut commands, &mut images, &assets, origin, angle, &turret);
    }
}

fn shortest_angle_difference(current: f32, target: f32) -> f32 {
    (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI
}

fn rotate_angle_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    current + shortest_angle_difference(current, target).clamp(-max_delta, max_delta)
}

fn closest_target(
    origin: Vec2,
    candidates: impl Iterator<Item = (Entity, Vec2)>,
    range: f32,
) -> Option<(Entity, Vec2)> {
    candidates
        .filter_map(|(entity, position)| {
            let distance_squared = origin.distance_squared(position);
            (distance_squared <= range * range).then_some((entity, position, distance_squared))
        })
        .min_by(|left, right| left.2.total_cmp(&right.2))
        .map(|(entity, position, _)| (entity, position))
}

fn fire_turret(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    assets: &GameAssets,
    origin: Vec2,
    angle: f32,
    turret: &Turret,
) {
    let direction = Vec2::from_angle(angle);
    let muzzle = origin + direction * turret.muzzle_offset;
    commands.spawn((
        Sprite {
            image: assets.bullet.clone(),
            custom_size: Some(Vec2::new(12.0, 5.0)),
            color: Color::srgb(0.55, 0.9, 1.0),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT)
            .with_rotation(Quat::from_rotation_z(angle)),
        Velocity(direction * turret.bullet_speed),
        Bullet {
            damage: turret.damage,
            radius: 4.0,
            friendly: true,
        },
        Lifetime(1.2),
        DespawnOnExit(GameState::Playing),
    ));
    commands.spawn((
        Sprite {
            image: assets.muzzle_flash.clone(),
            custom_size: Some(Vec2::splat(18.0)),
            color: Color::srgb(0.65, 0.9, 1.0),
            ..default()
        },
        Transform::from_xyz(muzzle.x, muzzle.y, Z_BRIGHT + 1.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        FadeOut {
            timer: 0.045,
            duration: 0.045,
        },
        DespawnOnExit(GameState::Playing),
    ));

    let eject = Vec2::from_angle(angle + std::f32::consts::FRAC_PI_2);
    let speed = 40.0 + fastrand::f32() * 45.0;
    spawn_casing(
        commands,
        images,
        assets.casing.clone(),
        origin,
        fastrand::f32() * std::f32::consts::TAU,
        eject * speed,
        0.5,
    );
}

fn update_turret_ammo_bars(
    mut commands: Commands,
    owners: Query<(&Transform, &Turret), Without<TurretAmmoBar>>,
    mut bars: Query<(Entity, &TurretAmmoBar, &mut Transform, &mut Sprite)>,
) {
    for (entity, bar, mut transform, mut sprite) in &mut bars {
        let Ok((owner_transform, turret)) = owners.get(bar.owner) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = owner_transform.translation.xy();
        let fraction = turret.ammo as f32 / turret.max_ammo as f32;
        transform.translation.x = position.x;
        transform.translation.y = position.y + bar.offset_y;
        if bar.fill {
            let width = bar.width * fraction.max(0.001);
            sprite.custom_size = Some(Vec2::new(width, 3.0));
            transform.translation.x = position.x - bar.width * 0.5 + width * 0.5;
            sprite.color = if fraction > 0.25 {
                Color::srgb(0.94, 0.68, 0.18)
            } else {
                Color::srgb(0.88, 0.2, 0.14)
            };
        } else {
            sprite.custom_size = Some(Vec2::new(bar.width + 4.0, 6.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        closest_target, rotate_angle_towards, shortest_angle_difference,
        turret_position_next_to_player, update_turret_ammo_bars, update_turrets,
    };
    use bevy::prelude::{App, IntoScheduleConfigs, Update, Vec2, World};

    #[test]
    fn turret_selects_the_closest_enemy_inside_its_range() {
        let mut world = World::new();
        let far = world.spawn_empty().id();
        let near = world.spawn_empty().id();
        let outside = world.spawn_empty().id();

        let target = closest_target(
            Vec2::ZERO,
            [
                (far, Vec2::new(80.0, 0.0)),
                (near, Vec2::new(20.0, 0.0)),
                (outside, Vec2::new(120.0, 0.0)),
            ]
            .into_iter(),
            100.0,
        );

        assert_eq!(target, Some((near, Vec2::new(20.0, 0.0))));
    }

    #[test]
    fn turret_is_placed_beside_the_player() {
        let player_position = Vec2::new(12.0, -8.0);
        let turret_position = turret_position_next_to_player(player_position, 0.0);

        assert!((turret_position.distance(player_position) - 40.0).abs() < 0.001);
        assert!((turret_position.x - player_position.x - 40.0).abs() < 0.001);
        assert!((turret_position.y - player_position.y).abs() < 0.001);
    }

    #[test]
    fn turret_system_queries_initialize_without_conflicts() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                update_turrets.run_if(|| false),
                update_turret_ammo_bars.run_if(|| false),
            ),
        );

        app.update();
    }

    #[test]
    fn turret_head_turns_at_a_limited_rate_across_angle_wraparound() {
        let current = std::f32::consts::PI - 0.1;
        let target = -std::f32::consts::PI + 0.1;
        let next = rotate_angle_towards(current, target, 0.05);

        assert!((shortest_angle_difference(current, next).abs() - 0.05).abs() < 0.001);
        assert!((shortest_angle_difference(next, target).abs() - 0.15).abs() < 0.001);
    }
}
