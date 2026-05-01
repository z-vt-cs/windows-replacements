use windows::Win32::UI::Shell::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use tauri::Manager;



fn register_appbar(hwnd: HWND) {
    let mut abd = APPBARDATA::default();
    abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    abd.hWnd = hwnd;

    let height = 40;
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };

    abd.uEdge = ABE_TOP;
    abd.rc.left = 0;
    abd.rc.top = 0;
    abd.rc.right = screen_width;
    abd.rc.bottom = height;

    unsafe {
        SHAppBarMessage(ABM_NEW, &mut abd);
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        SHAppBarMessage(ABM_SETPOS, &mut abd);

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            abd.rc.left,
            abd.rc.top,
            abd.rc.right - abd.rc.left,
            abd.rc.bottom - abd.rc.top,
            SWP_NOACTIVATE,
        );
    }
}

fn unregister_appbar(hwnd: HWND) {
    let mut abd = APPBARDATA::default();
    abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    abd.hWnd = hwnd;

    unsafe {
        SHAppBarMessage(ABM_REMOVE, &mut abd);
    }
}

// State we pass through LPARAM - can't capture in extern "system" fn
struct EnumState {
    windows: Vec<(HWND, String)>,
}

#[tauri::command]
fn get_taskbar_windows() -> Vec<String> {
    // collect only the String class names, drop the HWND
    let mut state = EnumState {
        windows: Vec::new(),
    };

    unsafe {
        // Pass a raw pointer to our state through LPARAM
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut state as *mut EnumState as isize),
        )
        .ok();
    }

    state.windows.into_iter().map(|(_hwnd, name)| name).collect()
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // Cast LPARAM back to our state
    let state = unsafe { &mut *(lparam.0 as *mut EnumState) };

    if is_taskbar_window(hwnd) {
        let mut class_name_buf = [0u16; 256];
        let length = unsafe { GetClassNameW(hwnd, &mut class_name_buf) } as usize;
        let class_name = String::from_utf16_lossy(&class_name_buf[..length]);

        state.windows.push((hwnd, class_name));
    }

    TRUE //keep enumerating
}

fn is_taskbar_window(hwnd: HWND) -> bool {
    unsafe {
        // 1. Must be visible
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

        // 2. WS_EX_TOOLWINDOW explicitly removes it from the taskbar
        if ex_style & WS_EX_TOOLWINDOW.0 != 0 {
            return false;
        }

        // 3. WS_EX_APPWINDOW forces it onto the taskbar
        if ex_style & WS_EX_APPWINDOW.0 != 0 {
            return true;
        }

        // 4. Skip windows that have a visible owner (e.g. dialogs)
        let owner = GetWindow(hwnd, GW_OWNER);
        if owner.0 != 0 && IsWindowVisible(owner).as_bool() {
            return false;
        }

        // 5. Skip child windows (has a parent)
        if GetParent(hwnd).0 != 0 {
            return false;
        }

        true
    }
}

fn get_window_icon(hwnd: HWND) -> Option<HICON> {
    let icon = unsafe { SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_SMALL as usize), LPARAM(0)) };
    if icon.0 != 0 {
        return Some(HICON(icon.0 as isize));
    }
    None
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::HasWindowHandle;
                if let Ok(handle) = window.window_handle() {
                    if let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                        let hwnd = HWND(win32.hwnd.get() as isize);
                        register_appbar(hwnd);                    }
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                #[cfg(target_os = "windows")]
                {
                    use raw_window_handle::HasWindowHandle;
                    if let Ok(handle) = window.window_handle() {
                        if let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
                            let hwnd = HWND(win32.hwnd.get() as isize);
                            unregister_appbar(hwnd);
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![get_taskbar_windows])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}