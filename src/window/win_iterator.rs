use anyhow::Result;
use windows::{
    Win32::{
        Foundation::{HWND, MAX_PATH},
        System::Console::GetConsoleWindow,
        UI::WindowsAndMessaging::{
            FindWindowExW, GW_CHILD, GW_HWNDNEXT, GetClassNameW, GetDesktopWindow, GetWindow,
            GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    },
    core::PWSTR,
};
use windows_result::Error;

use crate::utils::ToUtf8String;

use super::validator::{WindowSearchMode, is_window_valid};

// use crate::{
//     ProcessInfo, get_thread_proc_id,
//     validators::{WindowSearchMode, is_window_valid},
//     window::get_window_class,
// };

pub struct ProcessInfo {
    pub process_id: u32,
    pub thread_id: u32,
}

pub fn get_thread_proc_id(wnd: HWND) -> windows_result::Result<ProcessInfo> {
    let mut proc_id = 0u32;

    let thread_id = unsafe { GetWindowThreadProcessId(wnd, Some(&mut proc_id)) };
    if thread_id == 0 {
        return Err(Error::from_win32());
    }

    Ok(ProcessInfo {
        process_id: proc_id,
        thread_id,
    })
}

pub fn get_window_class(handle: HWND) -> anyhow::Result<String> {
    let mut class = [0_u16; MAX_PATH as usize + 1];

    let len = unsafe { GetClassNameW(handle, &mut class) };
    if len == 0 {
        return Err(Error::from_win32().into());
    }

    Ok(class.as_ref().to_utf8())
}

pub fn is_uwp_window(hwnd: HWND) -> Result<bool> {
    if hwnd.is_invalid() {
        return Ok(false);
    }

    let class = get_window_class(hwnd)?;
    Ok(class == "ApplicationFrameWindow")
}

pub fn get_uwp_actual_window(parent: HWND) -> Result<Option<HWND>> {
    let ProcessInfo {
        process_id: parent_id,
        ..
    } = get_thread_proc_id(parent)?;

    let mut child = unsafe { FindWindowExW(Some(parent), None, PWSTR::null(), PWSTR::null()) }?;

    while !child.is_invalid() {
        let ProcessInfo {
            process_id: child_id,
            ..
        } = get_thread_proc_id(child)?;

        if child_id != parent_id {
            return Ok(Some(child));
        }

        child = unsafe {
            FindWindowExW(Some(parent), Some(child), PWSTR::null(), PWSTR::null())
                .unwrap_or(HWND::default())
        };
    }

    Ok(None)
}

pub fn next_window(
    window: Option<HWND>,
    mode: WindowSearchMode,
    parent: &mut Option<HWND>,
    use_find_window_ex: bool,
) -> anyhow::Result<Option<HWND>> {
    let mut window = window.unwrap_or_default();

    let parent_valid = parent.is_some_and(|e| !e.is_invalid());
    if parent_valid {
        window = parent.unwrap_or_default();
        *parent = None;
    }

    loop {
        window = if use_find_window_ex {
            unsafe {
                FindWindowExW(
                    Some(GetDesktopWindow()),
                    Some(window),
                    PWSTR::null(),
                    PWSTR::null(),
                )
            }
        } else {
            unsafe { GetWindow(window, GW_HWNDNEXT) }
        }
        .unwrap_or(HWND::default());

        let valid = is_window_valid(window, mode).ok().unwrap_or(false);
        if window.is_invalid() || valid {
            break;
        }
    }

    let window_opt = if window.is_invalid() {
        None
    } else {
        Some(window)
    };

    if is_uwp_window(window)? {
        if format!("{:?}", window.0).ends_with("041098") {
            println!("UWP Window: {window:?}");
        }
        let actual = get_uwp_actual_window(window)?;
        if let Some(child) = actual {
            *parent = window_opt;

            return Ok(Some(child));
        }
    }

    Ok(window_opt)
}

pub fn first_window(
    mode: WindowSearchMode,
    parent: &mut Option<HWND>,
    use_find_window_ex: &mut bool,
) -> anyhow::Result<HWND> {
    let mut window =
        unsafe { FindWindowExW(Some(GetDesktopWindow()), None, PWSTR::null(), PWSTR::null()).ok() };

    if window.is_none() {
        *use_find_window_ex = false;
        window = unsafe { GetWindow(GetDesktopWindow(), GW_CHILD).ok() };
    } else {
        *use_find_window_ex = true;
    }

    *parent = None;

    let is_valid = window.is_some_and(|e| is_window_valid(e, mode).unwrap_or(false));

    if !is_valid {
        window = next_window(window, mode, parent, *use_find_window_ex)?;

        if window.is_none() && *use_find_window_ex {
            *use_find_window_ex = false;

            window = unsafe { GetWindow(GetDesktopWindow(), GW_CHILD).ok() };
            let valid = window.is_some_and(|e| is_window_valid(e, mode).unwrap_or(false));

            if !valid {
                window = next_window(window, mode, parent, *use_find_window_ex)?;
            }
        }
    }

    if window.is_none() {
        return Err(anyhow::anyhow!("No window found"));
    }

    let window = window.unwrap();
    if is_uwp_window(window)? {
        let child = get_uwp_actual_window(window)?;
        if let Some(c) = child {
            *parent = Some(window);
            return Ok(c);
        }
    }

    Ok(window)
}

#[derive(Debug)]
pub struct WindowInfo {
    pub title: String,
    pub hwnd: HWND,
}

fn get_window_title(handle: HWND) -> anyhow::Result<String> {
    let len = unsafe { GetWindowTextLengthW(handle) };
    if len == 0 {
        return Err(Error::from_win32().into());
    }

    let len = TryInto::<usize>::try_into(len)?;

    let mut title = vec![0_u16; len + 1];
    let get_title_res = unsafe { GetWindowTextW(handle, &mut title) };
    if get_title_res == 0 {
        return Err(Error::from_win32().into());
    }

    Ok(title.to_utf8())
}

fn get_window_info(hwnd: HWND) -> anyhow::Result<WindowInfo> {
    let title = get_window_title(hwnd)?;
    Ok(WindowInfo { title, hwnd })
}

pub fn get_all_windows(mode: WindowSearchMode) -> anyhow::Result<Vec<WindowInfo>> {
    let mut use_find_window_ex = false;

    let mut parent = None as Option<HWND>;
    let window = first_window(mode, &mut parent, &mut use_find_window_ex)?;
    let mut window = Some(window);

    let curr = unsafe { GetConsoleWindow() };

    let mut out = Vec::new();
    while window.is_some_and(|e| !e.is_invalid()) {
        let w = window.unwrap();
        if curr != w {
            let res = get_window_info(w);
            if let Ok(info) = res {
                out.push(info);
            } else {
                //eprintln!("Error: {:?}", res.err().unwrap());
            }
        }

        window = next_window(window, mode, &mut parent, use_find_window_ex)?;
    }

    Ok(out)
}
