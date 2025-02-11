use super::Event;
use crate::native_window_control_ffi::{self, safe_get_display_info};
use evdev::{AbsoluteAxisCode, Device, EventSummary, SynchronizationCode};
#[derive(Debug, Clone)]
pub struct FingerState {
    pub is_down: bool,
    pub pos: (f32, f32),
}

impl Default for FingerState {
    fn default() -> Self {
        Self {
            is_down: false,
            pos: (0.0, 0.0),
        }
    }
}
pub type MousePos = FingerState;
pub struct EventLoop {
    mouse_pos: std::sync::Arc<std::sync::RwLock<MousePos>>,
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

        //每秒轮询一次屏幕的朝向
        let realtime_orientation = std::sync::Arc::new(std::sync::RwLock::new(1 as u8));
        let realtime_orientation_clone = std::sync::Arc::clone(&realtime_orientation);
        std::thread::spawn(move || loop {
            if let Ok(mut ori) = realtime_orientation_clone.try_write() {
                let info = safe_get_display_info();
                *ori = info.orientation as u8;
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        });
        //更新触摸位置
        let mouse_pos = std::sync::Arc::new(std::sync::RwLock::new(MousePos::default()));
        let mouse_pos_clone = std::sync::Arc::clone(&mouse_pos);
        std::thread::spawn(move || {
            Self::refresh_mouse_pos(realtime_orientation, mouse_pos_clone, device);
        });

        Self { mouse_pos }
    }
    fn refresh_mouse_pos(
        realtime_orientation: std::sync::Arc<std::sync::RwLock<u8>>,
        pos: std::sync::Arc<std::sync::RwLock<MousePos>>,
        mut device: Device,
    ) {
        let mut finger_states: Vec<FingerState> = vec![];

        finger_states.resize_with(10, || FingerState {
            is_down: false,
            pos: (0.0, 0.0),
        });

        let mut least_finger_idx: usize = 0;

        //
        let mut phy_win_x:f32 = 0.0;
        let mut phy_win_y:f32 = 0.0;
        //
        for (code,absinfo) in device.get_absinfo().expect("can not get abs info of the touch"){
            match code {
                AbsoluteAxisCode::ABS_MT_POSITION_X=>{
                    phy_win_x = absinfo.maximum() as f32;
                },
                AbsoluteAxisCode::ABS_MT_POSITION_Y=>{
                    phy_win_y = absinfo.maximum() as f32;
                }
                _=>()
                
            }
        }  
        if phy_win_y == 0.0 || phy_win_x == 0.0{
            panic!("phy_win_x :{},phy_win_x:{},something went wrong!",phy_win_x,phy_win_y);
        } 

        let native_window_control_ffi::DisPlayInfo { width, height, .. } = safe_get_display_info();
        let mut screen_width: f32 = width as f32;
        let mut screen_height: f32 = height as f32;
        // 解决横屏的情况

        if screen_height < screen_width {
            std::mem::swap(&mut screen_height, &mut screen_width)
        }
        let scale_x = phy_win_x / screen_width;
        let scale_y = phy_win_y / screen_height;

        unsafe { crate::SCALE_FACTOR = scale_x / 10.0 }

        loop {
            // 读取输入事件
            for ev in device.fetch_events().expect("fetch_events failed!") {
                match ev.destructure() {
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_SLOT,value )=>{
                        least_finger_idx = value as usize;
                    },
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_TRACKING_ID,value )=>{
                        finger_states[least_finger_idx].is_down = value != -1;
                    },
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_X,value )=>{
                        finger_states[least_finger_idx].pos.0 = value as f32 / scale_x;
                    },
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_MT_POSITION_Y,value )=>{
                        finger_states[least_finger_idx].pos.1 = value as f32 / scale_y;
                    },
                    EventSummary::Synchronization(_, SynchronizationCode::SYN_REPORT,_ )=>{
                        if finger_states[least_finger_idx].is_down {
                            if let Ok(mut pos) = pos.try_write() {
                                if let Ok(ori) = realtime_orientation.try_read() {
                                    (*pos) = Self::touch_2_screen(
                                        screen_width,
                                        screen_height,
                                        *ori,
                                        finger_states[least_finger_idx].clone(),
                                    );
                                }
                            }
                        } else if let Ok(mut pos) = pos.try_write() {
                            pos.is_down = false;
                        }
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
        let x = phy_mouse_pos.pos.0;
        let y = phy_mouse_pos.pos.1;
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
        let mut mouse_cache = MousePos::default();
        loop {
            let now = std::time::Instant::now();

            let delta_time = now - last_frame;
            last_frame = now;
            if let Ok(pos) = self.mouse_pos.try_read() {
                mouse_cache.pos = pos.pos;
                mouse_cache.is_down = pos.is_down;
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
    device.supported_absolute_axes().is_some_and(|axes| {
        axes.contains(AbsoluteAxisCode::ABS_MT_POSITION_X)
            && axes.contains(AbsoluteAxisCode::ABS_MT_POSITION_Y)
    })
}
