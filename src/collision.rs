use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub enum Collider {
    Circle { radius: f32 },
    Aabb { half: Vec2 },
}

impl Collider {
    pub fn circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    pub fn aabb(half: Vec2) -> Self {
        Self::Aabb { half }
    }

    pub fn radius(&self) -> f32 {
        match self {
            Self::Circle { radius } => *radius,
            Self::Aabb { half } => half.length(),
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct StaticCollider(pub Collider);

#[cfg(test)]
pub fn circle_overlap(a: Vec2, ra: f32, b: Vec2, rb: f32) -> bool {
    let r = ra + rb;
    a.distance_squared(b) <= r * r
}

pub fn segment_circle_hit(a: Vec2, b: Vec2, center: Vec2, radius: f32) -> Option<Vec2> {
    let ab = b - a;
    let len_sq = ab.length_squared();
    let t = if len_sq <= f32::EPSILON {
        0.0
    } else {
        ((center - a).dot(ab) / len_sq).clamp(0.0, 1.0)
    };
    let closest = a + ab * t;
    if closest.distance_squared(center) <= radius * radius {
        Some(closest)
    } else {
        None
    }
}

pub fn circle_aabb_pushout(pos: Vec2, radius: f32, center: Vec2, half: Vec2) -> Option<Vec2> {
    let closest = Vec2::new(
        pos.x.clamp(center.x - half.x, center.x + half.x),
        pos.y.clamp(center.y - half.y, center.y + half.y),
    );
    let delta = pos - closest;
    let dist_sq = delta.length_squared();

    if dist_sq >= radius * radius {
        return None;
    }

    if dist_sq > f32::EPSILON {
        let dist = dist_sq.sqrt();
        return Some(delta / dist * (radius - dist));
    }

    let left = pos.x - (center.x - half.x);
    let right = (center.x + half.x) - pos.x;
    let bottom = pos.y - (center.y - half.y);
    let top = (center.y + half.y) - pos.y;
    let min = left.min(right).min(bottom).min(top);

    if min == left {
        Some(Vec2::new(-(left + radius), 0.0))
    } else if min == right {
        Some(Vec2::new(right + radius, 0.0))
    } else if min == bottom {
        Some(Vec2::new(0.0, -(bottom + radius)))
    } else {
        Some(Vec2::new(0.0, top + radius))
    }
}

pub fn segment_aabb_hit(a: Vec2, b: Vec2, center: Vec2, half: Vec2) -> bool {
    let min = center - half;
    let max = center + half;
    let d = b - a;
    let mut tmin = 0.0f32;
    let mut tmax = 1.0f32;

    for (origin, dir, lo, hi) in [(a.x, d.x, min.x, max.x), (a.y, d.y, min.y, max.y)] {
        if dir.abs() < f32::EPSILON {
            if origin < lo || origin > hi {
                return false;
            }
        } else {
            let inv = 1.0 / dir;
            let mut t1 = (lo - origin) * inv;
            let mut t2 = (hi - origin) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circles_overlap() {
        assert!(circle_overlap(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0));
        assert!(!circle_overlap(
            Vec2::ZERO,
            10.0,
            Vec2::new(21.0, 0.0),
            10.0
        ));
    }

    #[test]
    fn swept_segment_catches_fast_projectile() {
        let start = Vec2::new(-100.0, 0.0);
        let end = Vec2::new(100.0, 0.0);
        assert!(segment_circle_hit(start, end, Vec2::ZERO, 8.0).is_some());
        assert!(segment_circle_hit(start, end, Vec2::new(0.0, 30.0), 8.0).is_none());
    }

    #[test]
    fn circle_is_pushed_out_of_aabb() {
        let push = circle_aabb_pushout(Vec2::new(18.0, 0.0), 10.0, Vec2::ZERO, Vec2::splat(20.0));
        assert!(push.is_some());
        assert!(push.unwrap().x > 0.0);
    }

    #[test]
    fn circle_inside_aabb_gets_pushed_out() {
        let push = circle_aabb_pushout(Vec2::new(5.0, 0.0), 10.0, Vec2::ZERO, Vec2::splat(20.0));
        assert!(push.is_some());
        let pushed = Vec2::new(5.0, 0.0) + push.unwrap();
        assert!(pushed.x.abs() >= 20.0 + 10.0 - 0.01);
    }

    #[test]
    fn segment_hits_aabb() {
        assert!(segment_aabb_hit(
            Vec2::new(-100.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::ZERO,
            Vec2::splat(20.0)
        ));
        assert!(!segment_aabb_hit(
            Vec2::new(-100.0, 50.0),
            Vec2::new(100.0, 50.0),
            Vec2::ZERO,
            Vec2::splat(20.0)
        ));
    }
}
