pub mod event_loop;
pub enum Event {
    MouseMoving(f32, f32),
    MouseUp,
}
