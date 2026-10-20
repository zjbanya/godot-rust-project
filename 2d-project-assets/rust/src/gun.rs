use godot::classes::{Area2D, IArea2D, Marker2D, Node2D, Sprite2D};
use godot::prelude::*;

const WEAPON_PIVOT_PATH: &str = "WeaponPivot";
const PISTOL_PATH: &str = "WeaponPivot/Pistol";

/// 武器朝向控制器。
///
/// 职责只有两件事：
/// 1. 在 Area2D 的重叠体里找最近的 Mob；
/// 2. 让 WeaponPivot 指向它，并按左右方向翻转手枪贴图。
///
/// 不缓存目标：`get_overlapping_bodies()` 每帧返回当前有效的物理体，
/// 目标死亡或离开范围后自然消失，不需要 `is_instance_valid()`。
#[derive(GodotClass)]
#[class(base = Area2D)]
pub struct Gun {
    base: Base<Area2D>,

    /// 枪口旋转轴。它是 Gun 的子节点，生命周期确定，可以安全缓存。
    weapon_pivot: Option<Gd<Marker2D>>,

    /// 手枪贴图。Y 翻转作用在它身上。
    pistol: Option<Gd<Sprite2D>>,
}

#[godot_api]
impl IArea2D for Gun {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            weapon_pivot: None,
            pistol: None,
        }
    }

    fn ready(&mut self) {
        self.weapon_pivot = self.base().try_get_node_as::<Marker2D>(WEAPON_PIVOT_PATH);
        self.pistol = self.base().try_get_node_as::<Sprite2D>(PISTOL_PATH);

        if self.weapon_pivot.is_none() {
            godot_warn!("Gun 缺少子节点 `{WEAPON_PIVOT_PATH}`，无法旋转枪口");
        }
        if self.pistol.is_none() {
            godot_warn!("Gun 缺少子节点 `{PISTOL_PATH}`，无法执行 Y 翻转");
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.aim_at_nearest_mob();
    }
}

impl Gun {
    /// 在 Area2D 当前重叠的物理体中选择离枪口最近的一个。
    ///
    /// 候选体由 Gun 的 `collision_mask` 决定；当前场景配置为只检测
    /// `mob` 层（layer 2），所以这里不需要再按名字或类型过滤。
    fn find_nearest_mob(&self) -> Option<Gd<Node2D>> {
        let origin = self.weapon_pivot.as_ref().map_or_else(
            || self.base().get_global_position(),
            |pivot| pivot.get_global_position(),
        );

        let bodies = self.base().get_overlapping_bodies();
        let mut nearest: Option<(f32, Gd<Node2D>)> = None;

        for index in 0..bodies.len() {
            let Some(body) = bodies.get(index) else {
                continue;
            };

            let distance_squared = origin.distance_squared_to(body.get_global_position());
            let is_closer = nearest
                .as_ref()
                .map_or(true, |(best_distance, _)| distance_squared < *best_distance);

            if is_closer {
                nearest = Some((distance_squared, body));
            }
        }

        nearest.map(|(_, body)| body)
    }

    /// 指向最近的 Mob。没有目标时保持当前朝向，避免枪口突然回正。
    fn aim_at_nearest_mob(&mut self) {
        let Some(target) = self.find_nearest_mob() else {
            return;
        };
        let Some(mut pivot) = self.weapon_pivot.clone() else {
            return;
        };

        let target_position = target.get_global_position();
        let direction = target_position - pivot.get_global_position();

        // 目标与枪口重合时没有有效朝向，跳过这一帧。
        if direction.length_squared() < 1.0 {
            return;
        }

        pivot.look_at(target_position);

        // 目标在左侧时翻转贴图，避免手枪上下颠倒。
        if let Some(pistol) = self.pistol.as_mut() {
            pistol.set_flip_v(direction.x < 0.0);
        }
    }
}
