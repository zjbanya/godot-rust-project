use godot::{
    classes::{Area2D, IArea2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(base = Area2D)]
struct Bullet {
    base: Base<Area2D>,
}

#[godot_api]
impl IArea2D for Bullet {
    fn init(base: Base<Area2D>) -> Self {
        Self { base }
    }

    fn physics_process(&mut self, _delta: f64) {}

    fn ready(&mut self) {}
}
