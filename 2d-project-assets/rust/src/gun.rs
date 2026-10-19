use godot::{
    classes::{Area2D, IArea2D},
    prelude::*,
};

#[derive(GodotClass)]
#[class(base = Area2D)]
struct Gun {}

#[godot_api]
impl IArea2D for Gun {
    fn init(base: Base<Area2D>) -> Self {
        Self {}
    }

    fn physics_process(&mut self, _delta: f64) {}
}
