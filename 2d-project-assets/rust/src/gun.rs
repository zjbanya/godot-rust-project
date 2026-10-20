use godot::classes::{Area2D, IArea2D, Marker2D, Node, PackedScene, Sprite2D};
use godot::prelude::*;

use crate::bullet::Bullet;

const WEAPON_PIVOT_PATH: &str = "WeaponPivot";
const PISTOL_PATH: &str = "WeaponPivot/Pistol";
const SHOOTING_POINT_PATH: &str = "%ShootingPoint";

/// 子弹场景。Timer 的 timeout 触发时从这里实例化。
const BULLET_SCENE: &str = "res://bullet.tscn";

/// 武器控制器。
///
/// 职责：
/// 1. 每物理帧指向最近的 Mob；
/// 2. Timer.timeout 时在 ShootingPoint 生成一颗 Bullet。
#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct Gun {
    base: Base<Area2D>,

    weapon_pivot: Option<Gd<Marker2D>>,

    pistol: Option<Gd<Sprite2D>>,

    shooting_point: Option<Gd<Marker2D>>,

    bullet_scene: Option<Gd<PackedScene>>,
}

#[godot_api]
impl IArea2D for Gun {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            weapon_pivot: None,
            pistol: None,
            shooting_point: None,
            bullet_scene: None,
        }
    }

    fn ready(&mut self) {
        self.weapon_pivot = Some(self.base().get_node_as::<Marker2D>(WEAPON_PIVOT_PATH));
        self.pistol = Some(self.base().get_node_as::<Sprite2D>(PISTOL_PATH));
        self.shooting_point = Some(self.base().get_node_as::<Marker2D>(SHOOTING_POINT_PATH));
        self.bullet_scene = Some(load::<PackedScene>(BULLET_SCENE));
    }

    fn physics_process(&mut self, _delta: f64) {
        self.aim_at_nearest_mob();
    }
}

#[godot_api]
impl Gun {
    /// 由 gun.tscn 里的 Timer.timeout 信号调用。
    #[func]
    fn on_shoot_timeout(&mut self) {
        // 先保证枪口朝向当前目标，再按新方向生成子弹。
        self.aim_at_nearest_mob();

        let Some(target) = self.find_nearest_mob() else {
            return;
        };
        let (Some(point), Some(scene)) = (self.shooting_point.clone(), self.bullet_scene.clone())
        else {
            return;
        };

        let spawn_position = point.get_global_position();
        let direction = (target.get_global_position() - spawn_position).normalized();

        if direction == Vector2::ZERO {
            return;
        }

        let Some(node) = scene.instantiate() else {
            return;
        };
        let Ok(mut bullet) = node.try_cast::<Bullet>() else {
            return;
        };

        let Some(mut parent) = self
            .base()
            .get_tree()
            .get_current_scene()
            .or_else(|| self.base().get_parent())
        else {
            return;
        };

        parent.add_child(&bullet.clone().upcast::<Node>());
        bullet.set_global_position(spawn_position);
        bullet.bind_mut().launch(direction);
    }
}

impl Gun {
    fn find_nearest_mob(&self) -> Option<Gd<Node2D>> {
        let origin = if let Some(weapon_pivot) = self.weapon_pivot.as_ref() {
            weapon_pivot.get_global_position()
        } else {
            self.base().get_global_position()
        };

        let bodies = self.base().get_overlapping_bodies();
        let mut nearest: Option<(f32, Gd<Node2D>)> = None;

        for index in 0..bodies.len() {
            let Some(body) = bodies.get(index) else {
                continue;
            };

            let distance_squared = origin.distance_squared_to(body.get_global_position());

            let is_closer = if let Some((best_distance, _)) = nearest.as_ref() {
                distance_squared < *best_distance
            } else {
                true
            };

            if is_closer {
                nearest = Some((distance_squared, body));
            }
        }

        nearest.map(|(_, body)| body)
    }

    /// 指向最近的 Mob。没有目标时保持当前朝向。
    fn aim_at_nearest_mob(&mut self) {
        let Some(target) = self.find_nearest_mob() else {
            return;
        };
        let Some(mut pivot) = self.weapon_pivot.clone() else {
            return;
        };

        let target_position = target.get_global_position();
        let direction = target_position - pivot.get_global_position();

        if direction.length_squared() < 1.0 {
            return;
        }

        pivot.look_at(target_position);

        if let Some(pistol) = self.pistol.as_mut() {
            pistol.set_flip_v(direction.x < 0.0);
        }
    }
}
