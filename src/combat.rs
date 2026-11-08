use bevy::prelude::*;

use crate::arena::Arena;
use crate::collision::{
    segment_aabb_hit, segment_circle_hit, Collider, StaticCollider,
};
use crate::debug::DebugFlags;
use crate::enemy::Enemy;
use crate::fx::{spawn_impact, ScreenShake};
use crate::player::Player;
use crate::states::{gameplay_active, GameSet, GameState, Paused};

#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
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
}

#[derive(Message, Debug)]
pub struct Damage {
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message, Debug)]
pub struct EnemyKilled {
    pub position: Vec2,
    pub score: u32,
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
                    handle_player_death,
                    move_bullets,
                    tick_hit_flash,
                )
                    .chain()
                    .in_set(GameSet::Combat)
                    .run_if(gameplay_active),
            );
    }
}

fn bullet_collision(
    time: Res<Time>,
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Velocity, &Bullet)>,
    enemies: Query<(Entity, &Transform, &Collider), With<Enemy>>,
    statics: Query<(&Transform, &StaticCollider)>,
    mut damage: MessageWriter<Damage>,
    mut shake: ResMut<ScreenShake>,
) {
    let dt = time.delta_secs();

    for (bullet_entity, bullet_tf, velocity, bullet) in &bullets {
        let start = bullet_tf.translation.xy();
        let end = start + velocity.0 * dt;
        let mut consumed = false;

        for (enemy_entity, enemy_tf, collider) in &enemies {
            let radius = collider.radius() + bullet.radius;
            if let Some(point) =
                segment_circle_hit(start, end, enemy_tf.translation.xy(), radius)
            {
                damage.write(Damage {
                    target: enemy_entity,
                    amount: bullet.damage,
                });
                spawn_impact(&mut commands, point);
                shake.add(0.04);
                commands.entity(bullet_entity).despawn();
                consumed = true;
                break;
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
                spawn_impact(&mut commands, end);
                commands.entity(bullet_entity).despawn();
                break;
            }
        }
    }
}

fn apply_damage(
    mut damage_messages: MessageReader<Damage>,
    mut commands: Commands,
    mut healths: Query<(
        &mut Health,
        &mut HitFlash,
        &Transform,
        Option<&Enemy>,
        Option<&Player>,
    )>,
    mut killed: MessageWriter<EnemyKilled>,
    mut player_died: MessageWriter<PlayerDied>,
    mut shake: ResMut<ScreenShake>,
    debug: Option<Res<DebugFlags>>,
) {
    for message in damage_messages.read() {
        let Ok((mut health, mut flash, transform, enemy, player)) =
            healths.get_mut(message.target)
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
        flash.timer = 0.12;

        if health.current > 0.0 {
            continue;
        }

        let position = transform.translation.xy();
        if let Some(enemy) = enemy {
            killed.write(EnemyKilled {
                position,
                score: enemy.score,
            });
            commands.entity(message.target).despawn();
        } else if player.is_some() {
            player_died.write(PlayerDied);
            shake.add(0.8);
            commands.entity(message.target).despawn();
        }
    }
}

fn handle_player_death(
    mut died: MessageReader<PlayerDied>,
    mut next_state: ResMut<NextState<GameState>>,
    mut paused: ResMut<Paused>,
) {
    for _ in died.read() {
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
            || position.x.abs() > arena.half.x
            || position.y.abs() > arena.half.y
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
            sprite.color = Color::srgb(1.0, 0.45, 0.4);
        }
    }
}
