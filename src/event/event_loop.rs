use super::Event;
use crate::native_window_control_ffi::{self, safe_get_display_info};
use evdev::{AbsoluteAxisType, Device, InputEventKind, Synchronization};
#[derive(Debug, Clone)]
pub struct FingerState {
    pub is_down: bool,
    pub pos: (f32, f32),
}
impl FingerState {
    pub fn new() -> Self {
        Self {
            is_down: false,
            pos: (0.0, 0.0),
        }
    }
}
pub type MousePos = FingerState;
pub struct EventLoop {
    mouse_pos: std::sync::Arc<std::sync::Mutex<MousePos>>,
}

impl EventLoop {
    pub fn new() -> Self {
        let mut i = 0;
        let device: Option<Device> = loop {
            if let Ok(dev) = Device::open(format!("/dev/input/event{i}")) {
                if is_touch(&dev) {
                    break Some(dev);
                }
                i += 1;
                continue;
            } else {
                break None;
            }
        };

        let device = device.unwrap_or_else(|| panic!("can not find touch in your device"));
        // 打印设备名称
        println!("found touch: {}", device.name().unwrap_or("Unknown"));

        //每秒轮询一次屏幕的朝向
        let realtime_orientation = std::sync::Arc::new(std::sync::Mutex::new(1 as u8));
        let realtime_orientation_clone = std::sync::Arc::clone(&realtime_orientation);
        std::thread::spawn(move || loop {
            if let Ok(mut ori) = realtime_orientation_clone.try_lock() {
                let info = safe_get_display_info();
                *ori = info.orientation as u8;
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        });
        //更新触摸位置
        let mouse_pos = std::sync::Arc::new(std::sync::Mutex::new(MousePos::new()));
        let mouse_pos_clone = std::sync::Arc::clone(&mouse_pos);
        std::thread::spawn(move || {
            Self::refresh_mouse_pos(realtime_orientation, mouse_pos_clone, device);
        });

        Self { mouse_pos }
    }
    fn refresh_mouse_pos(
        realtime_orientation: std::sync::Arc<std::sync::Mutex<u8>>,
        pos: std::sync::Arc<std::sync::Mutex<MousePos>>,
        mut device: Device,
    ) {
        let mut finger_states: Vec<FingerState> = vec![];

        finger_states.resize_with(10, || FingerState {
            is_down: false,
            pos: (0.0, 0.0),
        });

        let mut least_finger_idx: usize = 0;

        //
        let infos = device.get_abs_state().expect("can not get abs info");
        let phy_win_x = infos[0].maximum as f32;
        let phy_win_y = infos[1].maximum as f32;
        //
        let native_window_control_ffi::DisPlayInfo { width, height, .. } = safe_get_display_info();
        let mut screen_width: f32 = width as f32;
        let mut screen_height: f32 = height as f32;
        // 解决横屏的情况

        if screen_height < screen_width {
            std::mem::swap(&mut screen_height, &mut screen_width)
        }
        let scale_x = phy_win_x / screen_width;
        let scale_y = phy_win_y / screen_height;
        loop {
            // 读取输入事件
            for ev in device.fetch_events().expect("fetch_events failed!") {
                match ev.kind() {
                    InputEventKind::AbsAxis(axis) => match axis {
                        AbsoluteAxisType::ABS_MT_SLOT => {
                            least_finger_idx = (ev.code() as usize).min(0).max(9);
                        }
                        AbsoluteAxisType::ABS_MT_TRACKING_ID => {
                            if ev.value() == -1 {
                                finger_states[least_finger_idx].is_down = false;
                            } else {
                                finger_states[least_finger_idx].is_down = true;
                            }
                        }
                        AbsoluteAxisType::ABS_MT_POSITION_X => {
                            finger_states[least_finger_idx].pos.0 = ev.value() as f32 / scale_x;
                        }
                        AbsoluteAxisType::ABS_MT_POSITION_Y => {
                            finger_states[least_finger_idx].pos.1 = ev.value() as f32 / scale_y;
                        }
                        _ => {}
                    },
                    InputEventKind::Synchronization(syn) => match syn {
                        Synchronization::SYN_REPORT => {
                            if finger_states[least_finger_idx].is_down {
                                if let Ok(mut pos) = pos.try_lock() {
                                    if let Ok(ori) = realtime_orientation.try_lock() {
                                        (*pos) = Self::touch_2_screen(
                                            screen_width,
                                            screen_height,
                                            *ori,
                                            finger_states[least_finger_idx].clone(),
                                        );
                                    }
                                }
                            } else {
                                if let Ok(mut pos) = pos.try_lock() {
                                    (*pos).is_down = finger_states[least_finger_idx].is_down;
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn touch_2_screen(
        screen_width: f32,
        screen_height: f32,
        realtime_orientation: u8,
        phy_mouse_pos: MousePos,
    ) -> MousePos {
        let mut phy_mouse_pos = phy_mouse_pos;
        let x = phy_mouse_pos.pos.0.clone();
        let y = phy_mouse_pos.pos.1.clone();
        match realtime_orientation {
            1 => {
                phy_mouse_pos.pos.0 = y;
                phy_mouse_pos.pos.1 = screen_width - x;
            }
            3 => {
                phy_mouse_pos.pos.1 = x;
                phy_mouse_pos.pos.0 = screen_height - y;
            }
            _ => {}
        }
        phy_mouse_pos
    }

    pub fn run<F>(self, mut event_handler: F)
    where
        F: FnMut(Event, std::time::Duration, &mut bool),
    {
        let mut run = true;
        let mut last_frame = std::time::Instant::now();
        let mut mouse_cache = MousePos::new();
        loop {
            let now = std::time::Instant::now();

            let delta_time = now - last_frame;
            last_frame = now;
            if let Ok(pos) = self.mouse_pos.try_lock() {
                mouse_cache.pos = (*pos).pos;
                mouse_cache.is_down = (*pos).is_down;
            }
            if mouse_cache.is_down {
                event_handler(
                    Event::MouseMoving(mouse_cache.pos.0, mouse_cache.pos.1),
                    delta_time,
                    &mut run,
                );
            } else {
                event_handler(Event::MouseUp, delta_time, &mut run);
            }

            if !run {
                break;
            }
        }
    }
}

fn is_touch(device: &Device) -> bool {
    return device.supported_absolute_axes().map_or(false, |axes| {
        axes.contains(AbsoluteAxisType::ABS_X)
            && axes.contains(AbsoluteAxisType::ABS_MT_SLOT)
            && axes.contains(AbsoluteAxisType::ABS_Y)
    });
}
