use godot::classes::{CanvasLayer, INode2D, Node, Node2D, PackedScene, PathFollow2D};
use godot::prelude::*;

use crate::mob::Mob;

const MOB_SCENE: &str = "res://mob.tscn";
const PATH_FOLLOW_PATH: &str = "%PathFollow2D";
const GAME_OVER_PATH: &str = "GameOver";

/// 主场景控制器：按 Timer 的节奏，在 PathFollow2D 的随机位置生成 Mob。
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Game {
    base: Base<Node2D>,

    path_follow: Option<Gd<PathFollow2D>>,
    mob_scene: Option<Gd<PackedScene>>,
    game_over: Option<Gd<CanvasLayer>>,
}

#[godot_api]
impl INode2D for Game {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            path_follow: None,
            mob_scene: None,
            game_over: None,
        }
    }

    fn ready(&mut self) {
        self.path_follow = Some(self.base().get_node_as::<PathFollow2D>(PATH_FOLLOW_PATH));
        self.mob_scene = Some(load::<PackedScene>(MOB_SCENE));
        self.game_over = Some(self.base().get_node_as::<CanvasLayer>(GAME_OVER_PATH));
    }
}

#[godot_api]
impl Game {
    /// 由 Player.health_depleted 信号调用。
    #[func]
    fn on_player_health_depleted(&mut self) {
        if let Some(mut game_over) = self.game_over.clone() {
            game_over.set_visible(true);
        }

        // 暂停整棵场景树，Player、Mob 和生成 Timer 都会停止。
        self.base().get_tree().set_pause(true);
    }

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
