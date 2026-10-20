use godot::classes::{Area2D, IArea2D, Node2D};
use godot::prelude::*;

use crate::mob::Mob;

/// 子弹。
///
/// 职责：
/// 1. 沿发射方向飞行，超过有效射程后销毁；
/// 2. 通过 `body_entered` 信号命中目标；
/// 3. 命中 Mob 时调用它的 `take_damage`。
#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct Bullet {
    base: Base<Area2D>,

    /// 飞行速度，单位像素/秒。
    #[export]
    speed: f32,

    /// 有效射程，单位像素。
    #[export]
    range: f32,

    /// 命中伤害。
    #[export]
    damage: f32,

    direction: Vector2,
    traveled: f32,
}

#[godot_api]
impl IArea2D for Bullet {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            speed: 350.0,
            range: 1200.0,
            damage: 10.0,
            direction: Vector2::ZERO,
            traveled: 0.0,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.direction == Vector2::ZERO {
            return;
        }

        let step = self.speed * delta as f32;
        self.traveled += step;

        if self.traveled >= self.range {
            self.base_mut().queue_free();
            return;
        }

        let motion = self.direction * step;
        let mut base = self.base_mut();
        let next_position = base.get_global_position() + motion;
        base.set_global_position(next_position);
    }
}

#[godot_api]
impl Bullet {
    /// 由 Gun 在生成子弹后调用。
    pub fn launch(&mut self, direction: Vector2) {
        let direction = direction.normalized();
        if direction == Vector2::ZERO {
            return;
        }

        self.direction = direction;
        self.traveled = 0.0;
        self.base_mut().set_rotation(direction.angle());
    }

    /// 对应 bullet.tscn 里的 `body_entered` 信号连接。
    ///
    /// 该连接由场景文件完成，方法暴露给 Godot 调用，不需要在 Rust 里
    /// 手动构造 Callable，也不会产生额外的生命周期问题。
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if self.base().is_queued_for_deletion() {
            return;
        }

        // 命中 Mob：按类型转换后调用它的受伤方法。
        if let Ok(mut mob) = body.clone().try_cast::<Mob>() {
            mob.bind_mut().take_damage(self.damage);
            self.base_mut().queue_free();
            return;
        }

        // 其他物理体（墙、树等）视为阻挡，子弹销毁。
        self.base_mut().queue_free();
    }
}
