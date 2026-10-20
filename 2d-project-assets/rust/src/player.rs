use godot::classes::{Area2D, CharacterBody2D, ICharacterBody2D, Input, Node2D, ProgressBar};
use godot::prelude::*;

/// HappyBoo 是 Player 的直接子节点，路径相对于 Player。
const HAPPY_BOO_PATH: &str = "HappyBoo";
const HURT_BOX_PATH: &str = "HurtBox";
const PROGRESS_BAR_PATH: &str = "ProgressBar";

/// GDScript 里的方法名。
const PLAY_WALK: &str = "play_walk_animation";
const PLAY_IDLE: &str = "play_idle_animation";

/// 玩家角色。
#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,

    #[export]
    speed: f32,

    #[export]
    max_health: f32,

    /// 单个 Slime 每秒造成的伤害。
    #[export]
    contact_damage_per_second: f32,

    health: f32,

    dead: bool,

    happy_boo: Option<Gd<Node2D>>,

    hurt_box: Option<Gd<Area2D>>,

    progress_bar: Option<Gd<ProgressBar>>,

    is_walking: bool,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 130.0,
            max_health: 100.0,
            contact_damage_per_second: 4.0,
            health: 100.0,
            dead: false,
            happy_boo: None,
            hurt_box: None,
            progress_bar: None,
            is_walking: false,
        }
    }

    fn ready(&mut self) {
        self.health = self.max_health;

        self.happy_boo = Some(self.base().get_node_as::<Node2D>(HAPPY_BOO_PATH));
        self.hurt_box = Some(self.base().get_node_as::<Area2D>(HURT_BOX_PATH));
        self.progress_bar = Some(self.base().get_node_as::<ProgressBar>(PROGRESS_BAR_PATH));

        self.play_animation(PLAY_IDLE);
        self.update_health_ui();
    }

    fn physics_process(&mut self, delta: f64) {
        if self.dead {
            let mut base = self.base_mut();
            base.set_velocity(Vector2::ZERO);
            base.move_and_slide();
            return;
        }

        self.apply_contact_damage(delta);

        let direction =
            Input::singleton().get_vector("move_left", "move_right", "move_up", "move_down");

        let is_walking = !direction.is_zero_approx();
        if is_walking != self.is_walking {
            self.is_walking = is_walking;
            let method = if is_walking { PLAY_WALK } else { PLAY_IDLE };
            self.play_animation(method);
        }

        let velocity = direction * self.speed;

        let mut base = self.base_mut();
        base.set_velocity(velocity);
        base.move_and_slide();
    }
}

#[godot_api]
impl Player {
    /// 生命值归零时发出。GameOver 逻辑可以连接到这个信号。
    #[signal]
    fn health_depleted();

    /// 扣除生命值。HurtBox 的接触伤害和未来其他伤害源都走这里。
    #[func]
    pub fn take_damage(&mut self, amount: f32) {
        if self.dead || amount <= 0.0 {
            return;
        }

        self.health = (self.health - amount).max(0.0);
        self.update_health_ui();

        if self.health <= 0.0 {
            self.dead = true;

            if let Some(mut hurt_box) = self.hurt_box.clone() {
                hurt_box.set_monitoring(false);
            }

            self.signals().health_depleted().emit();
        }
    }
}

impl Player {
    /// 每个贴身 Slime 每帧持续造成伤害，总伤害 = 单怪 DPS × 数量。
    fn apply_contact_damage(&mut self, delta: f64) {
        let damage = {
            let Some(hurt_box) = self.hurt_box.as_ref() else {
                return;
            };

            let count = hurt_box.get_overlapping_bodies().len() as f32;
            if count <= 0.0 {
                return;
            }

            self.contact_damage_per_second * count * delta as f32
        };

        self.take_damage(damage);
    }

    /// 满血时隐藏血条，掉血后显示。
    fn update_health_ui(&mut self) {
        let Some(progress_bar) = self.progress_bar.as_mut() else {
            return;
        };

        let max_health = self.max_health.max(1.0);
        progress_bar.set_max(max_health as f64);
        progress_bar.set_value(self.health as f64);
        progress_bar.set_visible(self.health < self.max_health);
    }

    fn play_animation(&mut self, method: &str) {
        if let Some(happy_boo) = self.happy_boo.as_mut() {
            happy_boo.call(method, &[]);
        }
    }
}
