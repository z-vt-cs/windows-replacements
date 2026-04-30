use windows::Win32::UI::Shell::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use tauri::Manager;



fn register_appbar(hwnd: HWND) {
    let mut abd = APPBARDATA::default();
    abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    abd.hWnd = hwnd;

    let height = 50;
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}