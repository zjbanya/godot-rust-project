use godot::classes::{CharacterBody2D, ICharacterBody2D, Input};
use godot::prelude::*;

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
}

#[godot_api]
impl ICharacterBody2D for Player {
    /// 对应 GDScript 的 `_init()`，节点创建时调用一次。
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self { base, speed: 300.0 }
    }

    /// 对应 GDScript 的 `_physics_process(delta)`。
    ///
    /// 物理相关逻辑放这里，而不是 `_process`：
    /// 它以固定频率运行，移动和碰撞结果更稳定、可复现。
    fn physics_process(&mut self, _delta: f64) {
        // 一次拿到归一化后的输入向量：
        // 引擎会自动处理死区，并保证斜向移动不会比直线更快。
        let input =
            Input::singleton().get_vector("move_left", "move_right", "move_up", "move_down");

        // 速度单位是“像素/秒”，move_and_slide() 内部会按物理帧长换算，
        // 所以这里不需要再乘 delta。
        let desired_velocity = input * self.speed;

        let mut base = self.base_mut();
        base.set_velocity(desired_velocity);
        // 由 CharacterBody2D 负责沿墙滑动和处理碰撞。
        base.move_and_slide();
    }
}
