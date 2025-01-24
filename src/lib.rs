pub mod event;
mod native_window_control_ffi;

use event::Event;
use native_window_control_ffi::{safe_create_native_window, safe_get_display_info, DisPlayInfo};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
};
pub struct Window {
    window_handle: RawWindowHandle,
    display_handle: RawDisplayHandle,
    width: u32,
    height: u32,
}
impl Window {
    pub fn new(title: &str) -> Self {
        let display_handle = RawDisplayHandle::Android(AndroidDisplayHandle::new());
        let DisPlayInfo {
            orientation: _,
            width,
            height,
        } = safe_get_display_info();
        let res = if width > height { width } else { height };
        unsafe {
            let ptr = core::ptr::NonNull::new_unchecked(safe_create_native_window(
                title, res as i32, res as i32, true,
            ));
            let window_handle = AndroidNdkWindowHandle::new(ptr.cast());

            return Self {
                window_handle: RawWindowHandle::AndroidNdk(window_handle),
                display_handle,
                width: width.try_into().unwrap(),
                height: height.try_into().unwrap(),
            };
        };
    }
    pub fn handle_event(io: &mut imgui::Io, event: Event, delta_time: std::time::Duration) {
        match event {
            Event::MouseDown(x, y) => {
                io.add_mouse_pos_event([x, y]);
                io.add_mouse_button_event(imgui::MouseButton::Left, true);
            }
            Event::MouseUp => {
                io.add_mouse_button_event(imgui::MouseButton::Left, false);
            }
        }
        io.update_delta_time(delta_time);
    }
    pub fn get_width(&self) -> u32 {
        return self.width;
    }
    pub fn get_height(&self) -> u32 {
        return self.height;
    }
    pub fn display_handle(&self) -> RawDisplayHandle {
        return self.display_handle;
    }
    pub fn window_handle(&self) -> RawWindowHandle {
        return self.window_handle;
    }
}
