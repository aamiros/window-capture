use window_capture::window::{WindowSearchMode, win_iterator};

fn main() {
    let windows = win_iterator::get_all_windows(WindowSearchMode::ExcludeMinimized)
        .expect("Failed to get all windows");
    for window in &windows {
        println!("Window: {}", window.title);
    }
    let window = windows
        .into_iter()
        .find(|w| w.title.contains("Clash"))
        .unwrap();
    d3d11::device_create(window.hwnd).expect("Failed to create device");
}
