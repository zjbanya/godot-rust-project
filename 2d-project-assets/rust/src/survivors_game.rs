use godot::classes::{INode2D, Node, Node2D, PackedScene, PathFollow2D};
use godot::prelude::*;

use crate::mob::Mob;

const MOB_SCENE: &str = "res://mob.tscn";
const PATH_FOLLOW_PATH: &str = "%PathFollow2D";

/// 主场景控制器：按 Timer 的节奏，在 PathFollow2D 的随机位置生成 Mob。
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Game {
    base: Base<Node2D>,

    path_follow: Option<Gd<PathFollow2D>>,
    mob_scene: Option<Gd<PackedScene>>,
}

#[godot_api]
impl INode2D for Game {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            path_follow: None,
            mob_scene: None,
        }
    }

    fn ready(&mut self) {
        self.path_follow = Some(self.base().get_node_as::<PathFollow2D>(PATH_FOLLOW_PATH));
        self.mob_scene = Some(load::<PackedScene>(MOB_SCENE));
    }
}

#[godot_api]
impl Game {
    /// 由 survivors_game.tscn 里的 Timer.timeout 信号调用。
    #[func]
    fn spawn_mob(&mut self) {
        let Some(mut path_follow) = self.path_follow.clone() else {
            return;
        };

        // 每次生成前随机一个路径进度，拿到路径上的世界坐标。
        path_follow.set_progress_ratio(godot::global::randf() as f32);
        let spawn_position = path_follow.get_global_position();

        let Some(scene) = self.mob_scene.clone() else {
            return;
        };
        let Some(node) = scene.instantiate() else {
            return;
        };
        let Ok(mut mob) = node.try_cast::<Mob>() else {
            return;
        };

        self.base_mut().add_child(&mob.clone().upcast::<Node>());
        mob.set_global_position(spawn_position);
    }
}
