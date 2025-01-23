pub mod event;
mod native_window_control_ffi;

use event::Event;
use native_window_control_ffi::{safe_create_native_window, safe_get_display_info, DisPlayInfo};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
};
pub struct Window {}
impl Window {
    pub fn create_window() -> (RawWindowHandle, RawDisplayHandle) {
        let display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::empty());
        let mut window_handle = AndroidNdkWindowHandle::empty();
        let DisPlayInfo {
            orientation: _,
            width,
            height,
        } = safe_get_display_info();
        let res = if width > height { width } else { height };
        window_handle.a_native_window =
            safe_create_native_window("fuck light bi", res as i32, res as i32, true);

        (RawWindowHandle::AndroidNdk(window_handle), display_handle)
    }
    pub fn handle_event(io: &mut imgui::Io, event: Event) {
        match event {
            Event::MouseDown(x, y) => {
                io.add_mouse_pos_event([x, y]);
                io.add_mouse_button_event(imgui::MouseButton::Left, true);
            }
            Event::MouseUp => {
                io.add_mouse_button_event(imgui::MouseButton::Left, false);
            }
        }
    }
}
