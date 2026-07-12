use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use win32pointer_rs::monitors;
use win32pointer_rs::pointers::{self, devicestates, pointerstates, PenContact, TouchContact};
use windows::Win32::Graphics::Gdi::HMONITOR;

use serde::Serialize;
use windows::Win32::UI::Input::Pointer::POINTER_FLAG_SECONDBUTTON;

#[derive(Serialize, Clone)]
pub struct ServerInfo {
    pub ip: String,
    pub port: u16,
    pub url: String,
}

#[derive(Serialize, Clone)]
pub struct Settings {
    pub needs_auth: bool,
    pub himetric: bool,
    pub touch_input: bool,
    pub auto_fullscreen: bool,
    pub y_scaling: bool,
    pub monitor_name: String,
}

struct ActiveTouch {
    x: i32,
    y: i32,
    is_primary: bool,
}

pub struct BridgeState {
    // Devices
    pen_device: Option<pointers::pointer::Device>,
    touch_device: Option<pointers::pointer::Device>,

    // Settings
    pub needs_auth: bool,
    pub himetric: bool,
    pub touch_input: bool,
    pub auto_fullscreen: bool,

    // Monitor
    current_monitor: HMONITOR,

    // Authentication
    authenticated: HashSet<String>,
    pub pin: Option<u32>,

    // Calibration
    client_width: Option<f64>,
    client_height: Option<f64>,
    pub factor_x: f64,
    pub factor_y: f64,
    y_scaling: bool,

    // Touch state
    touch_contacts: HashMap<u32, ActiveTouch>,
    first_touch_id: Option<u32>,

    // Server info
    pub server_info: ServerInfo,
}

// HMONITOR is a raw handle, safe to send across threads when properly synchronized
unsafe impl Send for BridgeState {}
unsafe impl Sync for BridgeState {}

pub type SharedBridgeState = Arc<Mutex<BridgeState>>;

impl BridgeState {
    pub fn new(ip: String, port: u16) -> Self {
        let pen_device = pointers::create_pen().ok();
        let touch_device = pointers::create_touch().ok();

        if pen_device.is_none() {
            log::error!("Failed to create pen device");
        }
        if touch_device.is_none() {
            log::error!("Failed to create touch device");
        }

        let current_monitor = monitors::get_primary_monitor();

        let url = format!("http://{}:{}", ip, port);

        let pin = Some(rand::random_range(1111..=9999));

        Self {
            pen_device,
            touch_device,
            needs_auth: true,
            himetric: true,
            touch_input: true,
            auto_fullscreen: true,
            current_monitor,
            authenticated: HashSet::new(),
            pin,
            client_width: None,
            client_height: None,
            factor_x: 1.0,
            factor_y: 1.0,
            y_scaling: false,
            touch_contacts: HashMap::new(),
            first_touch_id: None,
            server_info: ServerInfo { ip, port, url },
        }
    }

    pub fn is_authenticated(&self, sid: &str) -> bool {
        !self.needs_auth || self.authenticated.contains(sid)
    }

    pub fn add_authenticated(&mut self, sid: &str) {
        self.authenticated.insert(sid.to_string());
    }

    pub fn remove_authenticated(&mut self, sid: &str) {
        self.authenticated.remove(sid);
    }

    pub fn generate_pin(&mut self) -> u32 {
        let pin = rand::random_range(1111..=9999);
        self.pin = Some(pin);
        pin
    }

    pub fn recalculate_calibration_factor(&mut self) {
        if let (Some(client_width), Some(client_height)) = (self.client_width, self.client_height) {
            if let Ok(pos) = monitors::get_monitor_position(self.current_monitor) {
                let monitor_width = (pos.2 - pos.0) as f64;
                let monitor_height = (pos.3 - pos.1) as f64;
                self.factor_x = monitor_width / client_width;
                self.factor_y = monitor_height / client_height;
            }
        }
    }

    pub fn set_client_size(&mut self, width: f64, height: f64) {
        self.client_width = Some(width);
        self.client_height = Some(height);
        self.recalculate_calibration_factor();
    }

    pub fn switch_monitor_to_foreground(&mut self) -> String {
        self.current_monitor = monitors::get_current_monitor();
        self.recalculate_calibration_factor();
        self.get_monitor_info_string()
    }

    pub fn get_monitor_info_string(&self) -> String {
        if let Ok(pos) = monitors::get_monitor_position(self.current_monitor) {
            let scale = monitors::get_monitor_scaling(self.current_monitor).unwrap_or(1.0);
            format!(
                "{}x{} @ {:.0}% scale",
                pos.2 - pos.0,
                pos.3 - pos.1,
                scale * 100.0
            )
        } else {
            "Unknown monitor".to_string()
        }
    }

    fn scale_coords(&self, x: f64, y: f64) -> (i32, i32) {
        let y_factor = if self.y_scaling { self.factor_y } else { self.factor_x };
        ((x * self.factor_x) as i32, (y * y_factor) as i32)
    }

    // Pen injection methods
    pub fn inject_pen_down(&self, x: f64, y: f64, pressure: f64, erase: bool, barrel: bool) {
        let dev = match &self.pen_device {
            Some(d) => d,
            None => return,
        };

        let (sx, sy) = self.scale_coords(x, y);
        log::trace!("[INJECT] PEN_DOWN at x={}, y={} (scaled: {}, {}), pressure={}", x, y, sx, sy, pressure);
        let mut pointer_flags = pointerstates::pen::PEN_DOWN.0;
        let pen_flags = if erase {
            devicestates::Pen::PenEraser
        } else if barrel {
            pointer_flags |= POINTER_FLAG_SECONDBUTTON.0;
            devicestates::Pen::PenBarrel
        } else {
            devicestates::Pen::PenDefault
        };

        let contact = PenContact::new(sx, sy, (pressure * 1024.0) as u32, pen_flags, pointer_flags);
        let _ = pointers::inject_pen_contact_on_monitor(dev, self.current_monitor, &contact);
    }

    pub fn inject_pen_up(&self, x: f64, y: f64) {
        let dev = match &self.pen_device {
            Some(d) => d,
            None => return,
        };

        let (sx, sy) = self.scale_coords(x, y);
        log::trace!("[INJECT] PEN_UP at x={}, y={} (scaled: {}, {})", x, y, sx, sy);
        let contact = PenContact::new(
            sx, sy, 0,
            devicestates::Pen::PenDefault,
            pointerstates::pen::PEN_UP.0,
        );
        let _ = pointers::inject_pen_contact_on_monitor(dev, self.current_monitor, &contact);
    }

    pub fn inject_pen_hover(&self, x: f64, y: f64, erase: bool, barrel: bool) {
        let dev = match &self.pen_device {
            Some(d) => d,
            None => return,
        };

        let (sx, sy) = self.scale_coords(x, y);
        log::trace!("[INJECT] PEN_HOVER at x={}, y={} (scaled: {}, {})", x, y, sx, sy);
        let mut pointer_flags = pointerstates::pen::PEN_HOVER.0;
        let pen_flags = if erase {
            devicestates::Pen::PenInverted
        } else if barrel {
            pointer_flags |= POINTER_FLAG_SECONDBUTTON.0;
            devicestates::Pen::PenBarrel
        } else {
            devicestates::Pen::PenDefault
        };

        let contact = PenContact::new(sx, sy, 0, pen_flags, pointer_flags);
        let _ = pointers::inject_pen_contact_on_monitor(dev, self.current_monitor, &contact);
    }

    pub fn inject_pen_contact(
        &self, x: f64, y: f64, pressure: f64, erase: bool, barrel: bool,
        tilt_x: f64, tilt_y: f64,
    ) {
        let dev = match &self.pen_device {
            Some(d) => d,
            None => return,
        };

        let (sx, sy) = self.scale_coords(x, y);
        log::trace!("[INJECT] PEN_CONTACT at x={}, y={} (scaled: {}, {}), pressure={}", x, y, sx, sy, pressure);
        // The original uses TOUCH_CONTACT flags for pen contact (continuous movement)
        let mut pointer_flags = pointerstates::pen::PEN_CONTACT.0;
        let pen_flags = if erase {
            devicestates::Pen::PenEraser
        } else if barrel {
            pointer_flags |= POINTER_FLAG_SECONDBUTTON.0;
            devicestates::Pen::PenBarrel
        } else {
            devicestates::Pen::PenDefault
        };

        let contact = PenContact::new(
            sx, sy,
            (pressure * 1024.0) as u32,
            pen_flags,
            pointer_flags,
        )
        .with_himetric(self.himetric)
        .with_tilt(tilt_x as i32, tilt_y as i32);

        let _ = pointers::inject_pen_contact_on_monitor(dev, self.current_monitor, &contact);
    }

    // Touch injection methods
    pub fn inject_touch_down(&mut self, id: u32, x: f64, y: f64) {
        if !self.touch_input {
            return;
        }
        let (sx, sy) = self.scale_coords(x, y);
        let is_primary = self.touch_contacts.is_empty();
        if is_primary {
            self.first_touch_id = Some(id);
        }

        self.touch_contacts.insert(id, ActiveTouch { x: sx, y: sy, is_primary });
        self.inject_touch_frame(id, pointerstates::touch::TOUCH_DOWN.0);
    }

    pub fn inject_touch_up(&mut self, id: u32, x: f64, y: f64) {
        let (sx, sy) = self.scale_coords(x, y);
        if let Some(contact) = self.touch_contacts.get_mut(&id) {
            contact.x = sx;
            contact.y = sy;
        }
        self.inject_touch_frame(id, pointerstates::touch::TOUCH_UP.0);
        self.touch_contacts.remove(&id);
        if self.first_touch_id == Some(id) {
            self.first_touch_id = None;
        }
    }

    pub fn inject_touch_contact(&mut self, id: u32, x: f64, y: f64) {
        if !self.touch_input {
            return;
        }
        let (sx, sy) = self.scale_coords(x, y);
        if let Some(contact) = self.touch_contacts.get_mut(&id) {
            contact.x = sx;
            contact.y = sy;
        }
        self.inject_touch_frame(id, pointerstates::touch::TOUCH_CONTACT.0);
    }

    fn inject_touch_frame(&self, changed_id: u32, changed_state: u32) {
        let dev = match &self.touch_device {
            Some(d) => d,
            None => return,
        };

        let contacts: Vec<TouchContact> = self
            .touch_contacts
            .iter()
            .map(|(&id, active)| {
                let flags = if id == changed_id {
                    changed_state
                } else {
                    pointerstates::touch::TOUCH_CONTACT.0
                };
                TouchContact::new(id, active.x, active.y, flags)
                    .with_himetric(self.himetric)
                    .with_primary(active.is_primary)
            })
            .collect();

        if !contacts.is_empty() {
            let _ = pointers::inject_touch_frame_on_monitor(
                dev,
                self.current_monitor,
                &contacts,
            );
        }
    }

    pub fn toggle_y_scaling(&mut self) -> bool {
        self.y_scaling = !self.y_scaling;
        self.y_scaling
    }

    pub fn get_settings(&self) -> Settings {
        Settings {
            needs_auth: self.needs_auth,
            himetric: self.himetric,
            touch_input: self.touch_input,
            auto_fullscreen: self.auto_fullscreen,
            y_scaling: self.y_scaling,
            monitor_name: self.get_monitor_info_string(),
        }
    }
}
