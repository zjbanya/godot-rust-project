use godot::classes::{CharacterBody2D, ICharacterBody2D, Node, Node2D, PackedScene};
use godot::prelude::*;

/// 玩家所在的分组。
const PLAYER_GROUP: &str = "player";

/// Slime 是 Mob 的直接子节点，层级里已经设置好。
const SLIME_PATH: &str = "Slime";

/// Slime 脚本暴露的方法名。
const PLAY_WALK: &str = "play_walk";
const PLAY_IDLE: &str = "play_idle";
const PLAY_HURT: &str = "play_hurt";

/// 死亡时生成的烟雾爆炸场景。
const SMOKE_EXPLOSION_SCENE: &str = "res://smoke_explosion/smoke_explosion.tscn";

/// hurt 动画的持续状态时间。只影响动画，不停止移动。
const HURT_TIME: f64 = 0.3;

#[derive(PartialEq, Eq, Clone, Copy)]
enum MobState {
    Idle,
    Walk,
    Hurt,
}

/// 近战小怪：发现玩家后持续追击，受击播放 hurt，生命归零后立即隐藏并释放。
#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Mob {
    base: Base<CharacterBody2D>,

    #[export]
    speed: f32,

    #[export]
    max_health: f32,

    health: f32,

    slime: Option<Gd<Node2D>>,

    state: MobState,
    hurt_timer: f64,
    dead: bool,

    smoke_explosion_scene: Option<Gd<PackedScene>>,
}

#[godot_api]
impl ICharacterBody2D for Mob {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            speed: 85.0,
            max_health: 30.0,
            health: 30.0,
            slime: None,
            state: MobState::Idle,
            hurt_timer: 0.0,
            dead: false,
            smoke_explosion_scene: None,
        }
    }

    fn ready(&mut self) {
        self.health = self.max_health;

        // 层级已固定，直接取节点；缺失时让错误尽早暴露。
        self.slime = Some(self.base().get_node_as::<Node2D>(SLIME_PATH));
        self.smoke_explosion_scene = Some(load::<PackedScene>(SMOKE_EXPLOSION_SCENE));

        self.play_animation(PLAY_IDLE);
    }

    fn physics_process(&mut self, delta: f64) {
        if self.dead {
            return;
        }

        let target = self.find_target();

        if self.hurt_timer > 0.0 {
            self.hurt_timer -= delta;

            if self.hurt_timer <= 0.0 {
                let next_state = if target.is_some() {
                    MobState::Walk
                } else {
                    MobState::Idle
                };
                self.set_state(next_state);
            }
        } else {
            let next_state = if target.is_some() {
                MobState::Walk
            } else {
                MobState::Idle
            };
            self.set_state(next_state);
        }

        // 受击期间也继续移动，不再停顿。
        let mut velocity = Vector2::ZERO;
        if let Some(target) = target {
            let offset = target.get_global_position() - self.base().get_global_position();

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

#[godot_api]
impl Mob {
    #[func]
    pub fn take_damage(&mut self, amount: f32) {
        if self.dead || amount <= 0.0 {
            return;
        }

        self.health = (self.health - amount).max(0.0);

        if self.health <= 0.0 {
            self.die();
        } else {
            self.hurt_timer = HURT_TIME;
            self.state = MobState::Hurt;
            self.play_animation(PLAY_HURT);
        }
    }
}

impl Mob {
    fn die(&mut self) {
        self.dead = true;

        // 先放烟雾，再隐藏本体；本体立即释放，不在烟雾里停留。
        self.spawn_death_effect();

        let mut base = self.base_mut();
        base.set_visible(false);
        base.set_collision_layer(0);
        base.set_collision_mask(0);
        base.queue_free();
    }

    fn spawn_death_effect(&self) {
        let Some(scene) = self.smoke_explosion_scene.clone() else {
            return;
        };
        let Some(node) = scene.instantiate() else {
            return;
        };
        let Ok(mut effect) = node.try_cast::<Node2D>() else {
            return;
        };

        let Some(mut parent) = self.base().get_parent() else {
            return;
        };

        parent.add_child(&effect.clone().upcast::<Node>());
        effect.set_global_position(self.base().get_global_position() + Vector2::new(0.0, -20.0));
    }

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
            MobState::Hurt => self.play_animation(PLAY_HURT),
        }
    }

    fn play_animation(&mut self, method: &str) {
        if let Some(slime) = self.slime.as_mut() {
            slime.call(method, &[]);
        }
    }

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
