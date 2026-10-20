use godot::classes::{CharacterBody2D, ICharacterBody2D, Input, Node2D};
use godot::prelude::*;

/// HappyBoo 是 Player 的直接子节点，路径相对于 Player。
const HAPPY_BOO_PATH: &str = "HappyBoo";

// GDScript 的方法存在于脚本实例里，Rust 只能通过方法名字符串在运行时查找。这也是动态调用的代价：方法名写错不会编译报错，只会在运行时出错
// 这是学习 godot-rust 成本，而不是性能收益
/// GDScript 里的方法名。集中成常量，避免字符串散落。
const PLAY_WALK: &str = "play_walk_animation";
const PLAY_IDLE: &str = "play_idle_animation";

/// 玩家角色。
///
/// `#[class(base = CharacterBody2D)]` 做两件事：
/// 1. 告诉 gdext 这个 Rust 结构体继承 Godot 的 `CharacterBody2D`；
/// 2. 把它注册成一个全局类，名字就是 `Player`，之后能在编辑器里直接使用。
#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Player {
    /// 必须持有基类，才能调用 `move_and_slide()` 等 Godot 方法。
    base: Base<CharacterBody2D>,

    /// `#[export]` 会把字段暴露到 Godot 检查器里，
    /// 调速度时改场景参数即可，不用重新编译 Rust。
    #[export]
    speed: f32,

    /// 缓存 HappyBoo 节点，避免每帧都做一次场景树查找。
    happy_boo: Option<Gd<Node2D>>,

    /// 记录上一帧是否在移动，用来做状态边沿检测。
    is_walking: bool,
}

#[godot_api]
impl ICharacterBody2D for Player {
    /// 对应 GDScript 的 `_init()`，节点创建时调用一次。
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 130.0,
            happy_boo: None,
            is_walking: false,
        }
    }

    /// 对应 GDScript 的 `_ready()`：节点进入场景树后调用一次。
    /// 在这里解析并缓存子节点，比每帧查找更划算。
    fn ready(&mut self) {
        self.happy_boo = self.base().try_get_node_as::<Node2D>(HAPPY_BOO_PATH);
        if self.happy_boo.is_none() {
            godot_warn!("Player 找不到子节点 `{HAPPY_BOO_PATH}`，动画调用将被跳过");
        }
        self.play_animation(PLAY_IDLE);
    }

    /// 对应 GDScript 的 `_physics_process(delta)`。
    ///
    /// 物理相关逻辑放这里，而不是 `_process`：
    /// 它以固定频率运行，移动和碰撞结果更稳定、可复现。
    fn physics_process(&mut self, _delta: f64) {
        // 一次拿到归一化后的输入向量：
        // 引擎会自动处理死区，并保证斜向移动不会比直线更快。
        let direction =
            Input::singleton().get_vector("move_left", "move_right", "move_up", "move_down");

        // 只在“开始移动 / 停止移动”的瞬间切动画。
        // 如果每帧都调用 play()，动画会被反复重置到第 0 帧。
        let is_walking = !direction.is_zero_approx();
        if is_walking != self.is_walking {
            self.is_walking = is_walking;
            let method = if is_walking { PLAY_WALK } else { PLAY_IDLE };
            self.play_animation(method);
        }

        // 速度单位是“像素/秒”，move_and_slide() 内部会按物理帧长换算，
        // 所以这里不需要再乘 delta。
        let velocity = direction * self.speed;

        let mut base = self.base_mut();
        base.set_velocity(velocity);
        // 由 CharacterBody2D 负责沿墙滑动和处理碰撞。
        base.move_and_slide();
    }
}

impl Player {
    /// 调用 HappyBoo 脚本里的无参方法。
    ///
    /// Rust 在编译期不知道 GDScript 有哪些方法，所以只能用 `call()`
    /// 传方法名字符串；名字写错只会在运行时暴露。
    fn play_animation(&mut self, method: &str) {
        if let Some(happy_boo) = self.happy_boo.as_mut() {
            happy_boo.call(method, &[]);
        }
    }
}
