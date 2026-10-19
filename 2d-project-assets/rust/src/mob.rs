use godot::classes::{CharacterBody2D, ICharacterBody2D, Node2D};
use godot::prelude::*;

/// 玩家所在的分组。Mob 通过分组找目标，而不是写死 `../Player` 路径。
const PLAYER_GROUP: &str = "player";

/// Slime 是 Mob 的直接子节点，负责动画表现。
const SLIME_PATH: &str = "Slime";

/// Slime 脚本暴露的方法名。
const PLAY_WALK: &str = "play_walk";
const PLAY_IDLE: &str = "play_idle";

/// Mob 的行为状态。
///
/// 状态机的好处：动画、速度、目标处理都由状态驱动，
/// 不会出现“每帧到处 if 判断、动画被反复重置”的散乱逻辑。
#[derive(PartialEq, Eq, Clone, Copy)]
enum MobState {
    Idle,
    Chase,
}

/// 近战小怪：发现玩家后持续追击。
///
/// 职责边界：
/// 1. Mob 只负责感知、决策和移动；
/// 2. 动画细节交给 Slime 的脚本；
/// 3. 数值通过 `#[export]` 暴露给场景配置。
#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Mob {
    base: Base<CharacterBody2D>,

    /// 移动速度，单位像素/秒。策划可以在检查器里直接调。
    #[export]
    speed: f32,

    /// 缓存的 Slime 表现节点。
    slime: Option<Gd<Node2D>>,

    /// 当前追击目标。缓存起来，目标失效时再重新查找。
    target: Option<Gd<Node2D>>,

    state: MobState,
}

#[godot_api] 
impl ICharacterBody2D for Mob {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 250.0,
            slime: None,
            target: None,
            state: MobState::Idle,
        }
    }

    fn ready(&mut self) {
        self.slime = self.base().try_get_node_as::<Node2D>(SLIME_PATH);
        if self.slime.is_none() {
            godot_warn!("Mob 找不到子节点 `{SLIME_PATH}`，动画调用将被跳过");
        }

        // 初始表现：待机。第一帧物理更新时，如果发现玩家会自动切到追击。
        self.play_animation(PLAY_IDLE);
        self.refresh_target();
    }

    fn physics_process(&mut self, _delta: f64) {
        // 目标可能死亡、被移除或重生，所以先确认缓存是否还有效。
        self.refresh_target();

        // 有目标就追击，没有就待机。状态切换只发生一次。
        let next_state = if self.target.is_some() {
            MobState::Chase
        } else {
            MobState::Idle
        };
        self.set_state(next_state);

        // 先算出目标位置，避免在可变借用 self 时还持有 target 的借用。
        let target_position = self
            .target
            .as_ref()
            .map(|target| target.get_global_position());

        let mut velocity = Vector2::ZERO;
        if self.state == MobState::Chase {
            if let Some(target_position) = target_position {
                let offset = target_position - self.base().get_global_position();

                // 距离足够远才归一化，避免零向量和贴近目标时的抖动。
                if offset.length_squared() > 1.0 {
                    let direction = offset.normalized();
                    self.face(direction.x);
                    velocity = direction * self.speed;
                }
            }
        }

        let mut base = self.base_mut();
        base.set_velocity(velocity);
        base.move_and_slide();
    }
}

impl Mob {
    /// 目标缓存失效时，从 `player` 分组里重新找一个。
    ///
    /// 不在每帧都查分组：正常追击时只做一次有效性检查，
    /// 玩家死亡或换人时才重新查询。
    fn refresh_target(&mut self) {
        let still_valid = self
            .target
            .as_ref()
            .is_some_and(|target| target.is_instance_valid());

        if still_valid {
            return;
        }

        self.target = self
            .base()
            .get_tree()
            .get_first_node_in_group(PLAYER_GROUP)
            .and_then(|node| node.try_cast::<Node2D>().ok());
    }

    fn set_state(&mut self, next: MobState) {
        if self.state == next {
            return;
        }

        self.state = next;
        match next {
            MobState::Chase => self.play_animation(PLAY_WALK),
            MobState::Idle => self.play_animation(PLAY_IDLE),
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

    /// 调用 Slime 脚本里的无参方法。
    ///
    /// 和玩家调用 HappyBoo 一样，这是运行时动态调用。
    /// 方法名统一放在常量里，GDScript 改名时只需改一处。
    fn play_animation(&mut self, method: &str) {
        if let Some(slime) = self.slime.as_mut() {
            slime.call(method, &[]);
        }
    }
}
