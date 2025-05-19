use d3d11::GsDevice;
use window_capture::window::{WindowSearchMode, win_iterator};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

fn main() {
    env_logger::init();
    // 初始化 COM 库
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .expect("Failed to initialize COM");
    }
    let windows = win_iterator::get_all_windows(WindowSearchMode::ExcludeMinimized)
        .expect("Failed to get all windows");
    for window in &windows {
        println!("Window: {}", window.title);
    }
    let window = windows
        .into_iter()
        .find(|w| w.title.contains("Clash"))
        .unwrap();
    // d3d11::device_create(window.hwnd).expect("Failed to create device");

    let device = GsDevice::new(0).expect("Failed to create device");

    // 释放 COM 库
    unsafe {
        CoUninitialize();
    }
}
