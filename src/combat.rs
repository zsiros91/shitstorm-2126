use bevy::prelude::*;

use crate::arena::{Arena, Z_BRIGHT};
use crate::collision::{Collider, StaticCollider, segment_aabb_hit, segment_circle_hit};
use crate::debug::DebugFlags;
use crate::enemy::Enemy;
use crate::fx::{FadeOut, ScreenShake, spawn_blood, spawn_impact};
use crate::player::Player;
use crate::states::{GameSet, GameState, Paused, gameplay_active};

#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct HitFlash {
    pub timer: f32,
    pub base: Color,
}

impl Default for HitFlash {
    fn default() -> Self {
        Self {
            timer: 0.0,
            base: Color::WHITE,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy, Debug)]
pub struct Lifetime(pub f32);

#[derive(Component, Clone, Copy, Debug)]
pub struct Bullet {
    pub damage: f32,
    pub radius: f32,
    /// `true` for player rounds (hit enemies), `false` for enemy rounds.
    pub friendly: bool,
}

#[derive(Message, Debug)]
pub struct Damage {
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message, Debug)]
pub struct EnemyKilled {
    pub position: Vec2,
}

#[derive(Message, Debug)]
pub struct PlayerDied;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Damage>()
            .add_message::<EnemyKilled>()
            .add_message::<PlayerDied>()
            .add_systems(
                Update,
                (
                    bullet_collision,
                    apply_damage,
                    handle_game_over,
                    move_bullets,
                    tick_hit_flash,
                )
                    .chain()
                    .in_set(GameSet::Combat)
                    .run_if(gameplay_active),
            );
    }
}

pub fn bullet_collision(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<crate::assets::GameAssets>,
    bullets: Query<(Entity, &Transform, &Velocity, &Bullet)>,
    enemies: Query<(Entity, &Transform, &Collider), With<Enemy>>,
    players: Query<(Entity, &Transform, &Collider), (With<Player>, Without<Bullet>)>,
    statics: Query<
        (&Transform, &StaticCollider),
        (Without<Enemy>, Without<Player>, Without<Bullet>),
    >,
    mut damage: MessageWriter<Damage>,
    mut shake: ResMut<ScreenShake>,
) {
    let dt = time.delta_secs();

    for (bullet_entity, bullet_tf, velocity, bullet) in &bullets {
        let start = bullet_tf.translation.xy();
        let end = start + velocity.0 * dt;
        let mut consumed = false;

        if bullet.friendly {
            for (enemy_entity, enemy_tf, collider) in &enemies {
                let radius = collider.radius() + bullet.radius;
                if let Some(point) =
                    segment_circle_hit(start, end, enemy_tf.translation.xy(), radius)
                {
                    damage.write(Damage {
                        target: enemy_entity,
                        amount: bullet.damage,
                    });
                    spawn_impact(&mut commands, &assets, point);
                    spawn_hit_marker(&mut commands, &assets, point);
                    shake.add(0.02);
                    commands.entity(bullet_entity).despawn();
                    consumed = true;
                    break;
                }
            }
        } else {
            for (player_entity, player_tf, collider) in &players {
                let radius = collider.radius() + bullet.radius;
                if segment_circle_hit(start, end, player_tf.translation.xy(), radius).is_some() {
                    damage.write(Damage {
                        target: player_entity,
                        amount: bullet.damage,
                    });
                    shake.add(0.06);
                    commands.entity(bullet_entity).despawn();
                    consumed = true;
                    break;
                }
            }
        }

        if consumed {
            continue;
        }

        for (static_tf, static_collider) in &statics {
            let blocked = match static_collider.0 {
                Collider::Circle { radius } => segment_circle_hit(
                    start,
                    end,
                    static_tf.translation.xy(),
                    radius + bullet.radius,
                )
                .is_some(),
                Collider::Aabb { half } => {
                    segment_aabb_hit(start, end, static_tf.translation.xy(), half)
                }
            };

            if blocked {
                spawn_impact(&mut commands, &assets, end);
                commands.entity(bullet_entity).despawn();
                break;
            }
        }
    }
}

fn apply_damage(
    mut damage_messages: MessageReader<Damage>,
    mut commands: Commands,
    assets: Res<crate::assets::GameAssets>,
    mut healths: Query<
        (
            &mut Health,
            &mut HitFlash,
            &Transform,
            Option<&Enemy>,
            Option<&Player>,
        ),
        Without<StaticCollider>,
    >,
    mut killed: MessageWriter<EnemyKilled>,
    mut player_died: MessageWriter<PlayerDied>,
    mut shake: ResMut<ScreenShake>,
    debug: Option<Res<DebugFlags>>,
) {
    for message in damage_messages.read() {
        let Ok((mut health, mut flash, transform, enemy, player)) = healths.get_mut(message.target)
        else {
            continue;
        };

        if health.current <= 0.0 {
            continue;
        }

        if player.is_some() && debug.as_ref().map(|d| d.god_mode).unwrap_or(false) {
            continue;
        }

        health.current = (health.current - message.amount).max(0.0);
        flash.timer = 0.1;

        if health.current > 0.0 {
            continue;
        }

        let position = transform.translation.xy();
        if enemy.is_some() {
            killed.write(EnemyKilled { position });
            spawn_blood(&mut commands, &assets, position);
            shake.add(0.1);
            commands.entity(message.target).despawn();
        } else if player.is_some() {
            player_died.write(PlayerDied);
            spawn_blood(&mut commands, &assets, position);
            shake.add(0.7);
            commands.entity(message.target).despawn();
        } else {
            commands.entity(message.target).despawn();
        }
    }
}

fn handle_game_over(
    mut died: MessageReader<PlayerDied>,
    mut next_state: ResMut<NextState<GameState>>,
    mut paused: ResMut<Paused>,
) {
    let mut over = false;
    for _ in died.read() {
        over = true;
    }
    if over {
        paused.0 = false;
        next_state.set(GameState::GameOver);
    }
}

fn move_bullets(
    time: Res<Time>,
    mut commands: Commands,
    arena: Res<Arena>,
    mut bullets: Query<(Entity, &mut Transform, &Velocity, &mut Lifetime)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, velocity, mut lifetime) in &mut bullets {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
        lifetime.0 -= dt;

        let position = transform.translation.xy();
        if lifetime.0 <= 0.0
            || position.x.abs() > arena.half.x + 80.0
            || position.y.abs() > arena.half.y + 80.0
        {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_hit_flash(time: Res<Time>, mut flashes: Query<(&mut HitFlash, &mut Sprite)>) {
    for (mut flash, mut sprite) in &mut flashes {
        if flash.timer <= 0.0 {
            continue;
        }

        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            sprite.color = flash.base;
        } else {
            sprite.color = Color::srgb(1.0, 0.42, 0.32);
        }
    }
}

/// Brief hit confirmation tick shown where a player round landed.
fn spawn_hit_marker(commands: &mut Commands, assets: &crate::assets::GameAssets, position: Vec2) {
    commands.spawn((
        Sprite {
            image: assets.hit_marker.clone(),
            custom_size: Some(Vec2::splat(13.0)),
            color: Color::srgb(1.0, 0.92, 0.8),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, Z_BRIGHT + 3.0),
        FadeOut {
            timer: 0.12,
            duration: 0.12,
        },
        DespawnOnExit(GameState::Playing),
    ));
}
