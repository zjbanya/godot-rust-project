use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = Node2D)]
struct Game {
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Game {
    fn init(base: Base<Node2D>) -> Self {
        Self { base }
    }
}
