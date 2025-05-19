use std::sync::{Arc, Mutex};

use d3d11::GsDevice;
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::{
        Direct3D11::D3D11_BOX,
        Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
        Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT},
        Gdi::{ClientToScreen, MONITOR_DEFAULTTONEAREST, MonitorFromWindow},
    },
    UI::WindowsAndMessaging::{GetClientRect, IsIconic},
};

mod utils;
pub mod window;

fn get_client_box(window: HWND, width: u32, height: u32, client_box: &mut D3D11_BOX) -> bool {
    let mut client_rect = RECT::default();
    let mut window_rect = RECT::default();
    let mut upper_left = POINT::default();

    let client_box_available = unsafe {
        !IsIconic(window).as_bool()
            && GetClientRect(window, &mut client_rect).is_ok()
            && client_rect.right > 0
            && client_rect.bottom > 0
            && DwmGetWindowAttribute(
                window,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut window_rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            )
            .is_ok()
            && ClientToScreen(window, &mut upper_left).as_bool()
    };

    if client_box_available {
        let left = if upper_left.x > window_rect.left {
            (upper_left.x - window_rect.left) as u32
        } else {
            0
        };
        client_box.left = left;
        let top = if upper_left.y > window_rect.top {
            (upper_left.y - window_rect.top) as u32
        } else {
            0
        };
        client_box.top = top;
        let mut texture_width = 1u32;
        if width > left {
            texture_width = (width - left).min(client_rect.right as u32);
        }
        let mut texture_height = 1u32;
        if height > top {
            texture_height = (height - top).min(client_rect.bottom as u32);
        }
        client_box.right = left + texture_width;
        client_box.bottom = top + texture_height;
        client_box.front = 0;
        client_box.back = 1;

        client_box.right <= width && client_box.bottom <= height
    } else {
        client_box_available
    }
}

pub struct WindowCatpure {
    gs_device: Arc<Mutex<GsDevice>>,
}

impl WindowCatpure {
    fn get_pixel_format(&self, window: HWND, force_sdr: bool) -> DXGI_FORMAT {
        let sdr_format = DXGI_FORMAT_B8G8R8A8_UNORM;
        if force_sdr {
            return sdr_format;
        }
        let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
        let is_hdr = self
            .gs_device
            .lock()
            .unwrap()
            .device_is_monitor_hdr(monitor)
            .unwrap_or(false);
        if is_hdr {
            DXGI_FORMAT_R16G16B16A16_FLOAT
        } else {
            sdr_format
        }
    }
}
