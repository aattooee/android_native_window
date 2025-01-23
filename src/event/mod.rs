pub mod event_loop;
pub enum Event {
    MouseUp,
    MouseDown(f32, f32),
}
