use godot::classes::{CharacterBody2D, ICharacterBody2D, Node2D};
use godot::prelude::*;

/// 玩家所在的分组。分组是“当前存活玩家”的唯一事实来源。
const PLAYER_GROUP: &str = "player";

/// Slime 是 Mob 的直接子节点，负责动画表现。
const SLIME_PATH: &str = "Slime";

/// Slime 脚本暴露的方法名。
const PLAY_WALK: &str = "play_walk";
const PLAY_IDLE: &str = "play_idle";

/// Mob 的行为状态。
#[derive(PartialEq, Eq, Clone, Copy)]
enum MobState {
    Idle,
    Walk,
}

/// 近战小怪：发现玩家后持续追击。
///
/// 这里刻意不缓存 `target: Gd<Node2D>`。
/// 玩家死亡、重生、切换角色都会让旧指针失效；
/// 与其每帧用 `is_instance_valid()` 兜底，不如每帧从分组里取当前存活目标。
#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Mob {
    base: Base<CharacterBody2D>,

    /// 移动速度，单位像素/秒。策划可以在检查器里直接调。
    #[export]
    speed: f32,

    /// 缓存的 Slime 表现节点。它和 Mob 同生共死，生命周期是确定的。
    slime: Option<Gd<Node2D>>,

    state: MobState,
}

#[godot_api]
impl ICharacterBody2D for Mob {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 85.0,
            slime: None,
            state: MobState::Idle,
        }
    }

    fn ready(&mut self) {
        self.slime = self.base().try_get_node_as::<Node2D>(SLIME_PATH);
        if self.slime.is_none() {
            godot_warn!("Mob 找不到子节点 `{SLIME_PATH}`，动画调用将被跳过");
        }

        self.play_animation(PLAY_IDLE);
    }

    fn physics_process(&mut self, _delta: f64) {
        // 只在这一帧内持有目标指针，不在帧与帧之间保存。
        // 玩家被移除后会自动从分组里消失，因此不会拿到失效指针。
        let target = self.find_target();

        let next_state = if target.is_some() {
            MobState::Walk
        } else {
            MobState::Idle
        };
        self.set_state(next_state);

        let mut velocity = Vector2::ZERO;
        if let Some(target) = target {
            let offset = target.get_global_position() - self.base().get_global_position();

            // 距离足够远才归一化，避免零向量和贴近目标时的抖动。
            if offset.length_squared() > 1.0 {
                let direction = offset.normalized();
                self.face(direction.x);
                velocity = direction * self.speed;
            }
        }

        let mut base = self.base_mut();
        base.set_velocity(velocity);
        base.move_and_slide();
    }
}

impl Mob {
    /// 从 `player` 分组取当前存活目标。
    ///
    /// 分组由场景树维护：节点被 `free()` 时会自动从分组移除。
    /// 这次查询的结果只在当前物理帧内使用，所以不需要 `is_instance_valid()`。
    fn find_target(&self) -> Option<Gd<Node2D>> {
        self.base()
            .get_tree()
            .get_first_node_in_group(PLAYER_GROUP)
            .and_then(|node| node.try_cast::<Node2D>().ok())
    }

    fn set_state(&mut self, next_state: MobState) {
        if self.state == next_state {
            return;
        }

        self.state = next_state;
        match next_state {
            MobState::Idle => self.play_animation(PLAY_IDLE),
            MobState::Walk => self.play_animation(PLAY_WALK),
        }
    }

    /// 调用 Slime 脚本里的无参方法。
    fn play_animation(&mut self, method: &str) {
        if let Some(slime) = self.slime.as_mut() {
            slime.call(method, &[]);
        }
    }

    /// 根据水平移动方向翻转视觉节点，同时保留美术原始缩放。
    fn face(&mut self, direction_x: f32) {
        if direction_x.abs() < 0.01 {
            return;
        }

        if let Some(slime) = self.slime.as_mut() {
            let mut scale = slime.get_scale();
            scale.x = scale.x.abs() * if direction_x < 0.0 { -1.0 } else { 1.0 };
            slime.set_scale(scale);
        }
    }
}
