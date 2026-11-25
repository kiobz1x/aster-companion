mod actions;
mod brain;
mod config;
mod docs;
mod games;
mod media;
mod memory;
mod skins;
mod voice;

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, PhysicalPosition, RunEvent, State, WebviewUrl, WebviewWindowBuilder};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;
const AUTOSTART_REG_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_REG_VAL: &str = "AsterCompanion";

fn is_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("reg");
        cmd.args(["query", AUTOSTART_REG_KEY, "/v", AUTOSTART_REG_VAL]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn set_autostart_enabled(enable: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("reg");
        if enable {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = format!("\"{}\"", exe.display());
            cmd.args([
                "add",
                AUTOSTART_REG_KEY,
                "/v",
                AUTOSTART_REG_VAL,
                "/t",
                "REG_SZ",
                "/d",
                &exe_str,
                "/f",
            ]);
        } else {
            cmd.args(["delete", AUTOSTART_REG_KEY, "/v", AUTOSTART_REG_VAL, "/f"]);
        }
        cmd.creation_flags(CREATE_NO_WINDOW);
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            Ok(enable)
        } else {
            Err("Falha ao atualizar registro de inicialização do Windows.".into())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        Err("Inicialização automática disponível apenas no Windows.".into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

struct HitState {
    rects: Arc<Mutex<Vec<HitRect>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopBounds {
    pub min_x: i32,
    pub max_x: i32,
    pub floor_y: i32,
    pub win_x: i32,
    pub win_y: i32,
    pub scale: f64,
}

#[tauri::command]
fn get_desktop_bounds(window: tauri::WebviewWindow) -> Result<DesktopBounds, String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "Monitor não encontrado".to_string())?;

    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;

    let margin_x = (12.0 * scale) as i32;
    let margin_y = (10.0 * scale) as i32;
    let min_x = work.position.x + margin_x;
    let max_x = (work.position.x + work.size.width as i32 - size.width as i32 - margin_x).max(min_x);
    let floor_y = work.position.y + work.size.height as i32 - size.height as i32 - margin_y;

    Ok(DesktopBounds {
        min_x,
        max_x,
        floor_y,
        win_x: pos.x,
        win_y: pos.y,
        scale,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct DragStartInfo {
    pub grab_dx: i32,
    pub grab_dy: i32,
    pub last_cx: i32,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DragPollResult {
    pub pressed: bool,
    pub cx: i32,
    pub cy: i32,
    pub vx: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerchTarget {
    pub snap_x: i32,
    pub snap_y: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoPerchTarget {
    pub hwnd: isize,
    pub snap_x: i32,
    pub snap_y: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveWindowContext {
    pub title: String,
    pub is_fullscreen: bool,
}

#[cfg(windows)]
mod win_drag {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetCursorPos(lp_point: *mut Point) -> i32;
        pub fn GetAsyncKeyState(v_key: i32) -> i16;
        pub fn EnumWindows(
            lp_enum_func: extern "system" fn(isize, isize) -> i32,
            l_param: isize,
        ) -> i32;
        pub fn IsWindowVisible(h_wnd: isize) -> i32;
        pub fn IsIconic(h_wnd: isize) -> i32;
        pub fn GetWindowRect(h_wnd: isize, lp_rect: *mut Rect) -> i32;
        pub fn GetWindowTextW(h_wnd: isize, lp_string: *mut u16, n_max_count: i32) -> i32;
        pub fn GetWindowThreadProcessId(h_wnd: isize, lpdw_process_id: *mut u32) -> u32;
        pub fn GetForegroundWindow() -> isize;
    }

    #[link(name = "dwmapi")]
    extern "system" {
        pub fn DwmGetWindowAttribute(
            h_wnd: isize,
            dw_attribute: u32,
            pv_attribute: *mut c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    pub fn cursor_pos() -> Option<(i32, i32)> {
        let mut pt = Point { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut pt) } != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }

    pub fn left_mouse_down() -> bool {
        let state = unsafe { GetAsyncKeyState(0x01) };
        ((state as u16) & 0x8000) != 0
    }

    pub fn find_perch_edge(
        self_pid: u32,
        center_x: i32,
        sit_line_y: i32,
        snap_dist: i32,
    ) -> Option<i32> {
        struct SearchCtx {
            self_pid: u32,
            center_x: i32,
            sit_line_y: i32,
            snap_dist: i32,
            best_top: Option<i32>,
        }

        extern "system" fn enum_cb(hwnd: isize, lparam: isize) -> i32 {
            let ctx = unsafe { &mut *(lparam as *mut SearchCtx) };
            unsafe {
                if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                    return 1;
                }
                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, &mut pid);
                if pid == ctx.self_pid {
                    return 1;
                }

                // Ignora janelas UWP ocultas/cloaked do Windows 11
                let mut cloaked: u32 = 0;
                let hr = DwmGetWindowAttribute(
                    hwnd,
                    14, // DWMWA_CLOAKED
                    (&mut cloaked as *mut u32).cast::<c_void>(),
                    std::mem::size_of::<u32>() as u32,
                );
                if hr == 0 && cloaked != 0 {
                    return 1;
                }

                let mut title_buf = [0u16; 128];
                let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
                if len <= 0 {
                    return 1;
                }
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);
                if title == "Program Manager" || title == "Windows Input Experience" {
                    return 1;
                }

                let mut rect = Rect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if GetWindowRect(hwnd, &mut rect) == 0 {
                    return 1;
                }

                let w = rect.right - rect.left;
                let h = rect.bottom - rect.top;
                if w < 260 || h < 160 || rect.top < 15 {
                    return 1;
                }

                if ctx.center_x >= rect.left + 40
                    && ctx.center_x <= rect.right - 40
                    && (rect.top - ctx.sit_line_y).abs() <= ctx.snap_dist
                {
                    ctx.best_top = Some(rect.top);
                    return 0; // Primeira janela no Z-order (mais acima) vence!
                }
            }
            1
        }

        let mut ctx = SearchCtx {
            self_pid,
            center_x,
            sit_line_y,
            snap_dist,
            best_top: None,
        };
        unsafe {
            EnumWindows(enum_cb, (&mut ctx as *mut SearchCtx) as isize);
        }
        ctx.best_top
    }

    pub fn find_auto_perch_window(
        self_pid: u32,
        min_top: i32,
        max_top: i32,
        min_left: i32,
        max_right: i32,
    ) -> Option<(isize, i32, i32, i32)> {
        struct AutoCtx {
            self_pid: u32,
            min_top: i32,
            max_top: i32,
            min_left: i32,
            max_right: i32,
            found: Option<(isize, i32, i32, i32)>,
        }

        extern "system" fn cb(hwnd: isize, lparam: isize) -> i32 {
            let ctx = unsafe { &mut *(lparam as *mut AutoCtx) };
            unsafe {
                if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                    return 1;
                }
                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, &mut pid);
                if pid == ctx.self_pid {
                    return 1;
                }

                let mut cloaked: u32 = 0;
                let hr = DwmGetWindowAttribute(
                    hwnd,
                    14,
                    (&mut cloaked as *mut u32).cast::<c_void>(),
                    std::mem::size_of::<u32>() as u32,
                );
                if hr == 0 && cloaked != 0 {
                    return 1;
                }

                let mut title_buf = [0u16; 128];
                let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
                if len <= 0 {
                    return 1;
                }
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);
                if title == "Program Manager" || title == "Windows Input Experience" {
                    return 1;
                }

                let mut rect = Rect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if GetWindowRect(hwnd, &mut rect) == 0 {
                    return 1;
                }

                let w = rect.right - rect.left;
                let h = rect.bottom - rect.top;
                if w >= 360
                    && h >= 220
                    && rect.top >= ctx.min_top
                    && rect.top <= ctx.max_top
                    && rect.right > ctx.min_left + 120
                    && rect.left < ctx.max_right - 120
                {
                    ctx.found = Some((hwnd, rect.left, rect.right, rect.top));
                    return 0;
                }
            }
            1
        }

        let mut ctx = AutoCtx {
            self_pid,
            min_top,
            max_top,
            min_left,
            max_right,
            found: None,
        };
        unsafe {
            EnumWindows(cb, (&mut ctx as *mut AutoCtx) as isize);
        }
        ctx.found
    }

    pub fn get_alive_window_top(hwnd: isize, min_top: i32) -> Option<i32> {
        unsafe {
            if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                return None;
            }
            let mut rect = Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return None;
            }
            if rect.top < min_top || (rect.right - rect.left) < 260 {
                return None;
            }
            Some(rect.top)
        }
    }

    pub fn get_foreground_info(
        self_pid: u32,
        mon_x: i32,
        mon_y: i32,
        mon_w: i32,
        mon_h: i32,
    ) -> (String, bool) {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 || IsWindowVisible(hwnd) == 0 {
                return (String::new(), false);
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == self_pid {
                return (String::new(), false);
            }

            let mut title_buf = [0u16; 256];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
            let title = if len > 0 {
                String::from_utf16_lossy(&title_buf[..len as usize])
            } else {
                String::new()
            };

            if title == "Program Manager" || title.is_empty() {
                return (String::new(), false);
            }

            let mut rect = Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            let is_fullscreen = if GetWindowRect(hwnd, &mut rect) != 0 {
                rect.left <= mon_x + 2
                    && rect.top <= mon_y + 2
                    && rect.right >= mon_x + mon_w - 2
                    && rect.bottom >= mon_y + mon_h - 2
            } else {
                false
            };

            (title, is_fullscreen)
        }
    }
}

#[tauri::command]
fn drag_start(window: tauri::WebviewWindow) -> Result<DragStartInfo, String> {
    #[cfg(windows)]
    {
        let (cx, cy) = win_drag::cursor_pos().ok_or_else(|| "Sem posição do cursor".to_string())?;
        let pos = window.outer_position().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().unwrap_or(1.0);
        Ok(DragStartInfo {
            grab_dx: cx - pos.x,
            grab_dy: cy - pos.y,
            last_cx: cx,
            scale,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Err("Drag físico disponível apenas no Windows.".into())
    }
}

#[tauri::command]
fn drag_poll(
    window: tauri::WebviewWindow,
    grab_dx: i32,
    grab_dy: i32,
    last_cx: i32,
) -> Result<DragPollResult, String> {
    #[cfg(windows)]
    {
        let (cx, cy) = win_drag::cursor_pos().ok_or_else(|| "Sem posição do cursor".to_string())?;
        let pressed = win_drag::left_mouse_down();
        let scale = window.scale_factor().unwrap_or(1.0);
        if pressed {
            let _ = window.set_position(PhysicalPosition::new(cx - grab_dx, cy - grab_dy));
        }
        Ok(DragPollResult {
            pressed,
            cx,
            cy,
            vx: (cx - last_cx) as f64 / scale,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = (window, grab_dx, grab_dy, last_cx);
        Err("Drag físico disponível apenas no Windows.".into())
    }
}

#[tauri::command]
fn find_window_perch(window: tauri::WebviewWindow) -> Result<Option<PerchTarget>, String> {
    #[cfg(windows)]
    {
        let pos = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().unwrap_or(1.0);

        let sit_offset_px = (382.0 * scale) as i32;
        let center_x = pos.x + (size.width as i32) / 2;
        let sit_line_y = pos.y + sit_offset_px;
        let snap_dist = (72.0 * scale) as i32;

        if let Some(edge_top) =
            win_drag::find_perch_edge(std::process::id(), center_x, sit_line_y, snap_dist)
        {
            return Ok(Some(PerchTarget {
                snap_x: pos.x,
                snap_y: edge_top - sit_offset_px,
            }));
        }
        Ok(None)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Ok(None)
    }
}

#[tauri::command]
fn find_auto_perch(window: tauri::WebviewWindow) -> Result<Option<AutoPerchTarget>, String> {
    #[cfg(windows)]
    {
        let monitor = window
            .current_monitor()
            .map_err(|e| e.to_string())?
            .or_else(|| window.primary_monitor().ok().flatten())
            .ok_or_else(|| "Monitor não encontrado".to_string())?;
        let work = monitor.work_area();
        let scale = monitor.scale_factor();
        let size = window.outer_size().map_err(|e| e.to_string())?;

        let sit_offset_px = (382.0 * scale) as i32;
        // Garante espaço para a cabeça da Aster acima da borda da janela
        let min_top = work.position.y + (220.0 * scale) as i32;
        let max_top = work.position.y + work.size.height as i32 - (160.0 * scale) as i32;

        if let Some((hwnd, left, right, top)) = win_drag::find_auto_perch_window(
            std::process::id(),
            min_top,
            max_top,
            work.position.x,
            work.position.x + work.size.width as i32,
        ) {
            // Senta na parte direita da borda (sem tapar o X de fechar)
            let desired_center = left + ((right - left) as f64 * 0.65) as i32;
            let min_x = work.position.x + (12.0 * scale) as i32;
            let max_x = (work.position.x + work.size.width as i32 - size.width as i32 - (12.0 * scale) as i32).max(min_x);
            let snap_x = (desired_center - (size.width as i32) / 2).clamp(min_x, max_x);
            let snap_y = top - sit_offset_px;
            return Ok(Some(AutoPerchTarget {
                hwnd,
                snap_x,
                snap_y,
            }));
        }
        Ok(None)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Ok(None)
    }
}

#[tauri::command]
fn check_perch_alive(window: tauri::WebviewWindow, hwnd: isize) -> Result<Option<i32>, String> {
    #[cfg(windows)]
    {
        let scale = window.scale_factor().unwrap_or(1.0);
        let sit_offset_px = (382.0 * scale) as i32;
        let min_top = (140.0 * scale) as i32;
        if let Some(top) = win_drag::get_alive_window_top(hwnd, min_top) {
            return Ok(Some(top - sit_offset_px));
        }
        Ok(None)
    }
    #[cfg(not(windows))]
    {
        let _ = (window, hwnd);
        Ok(None)
    }
}

#[tauri::command]
fn get_active_window_context(window: tauri::WebviewWindow) -> Result<ActiveWindowContext, String> {
    #[cfg(windows)]
    {
        let monitor = window
            .current_monitor()
            .ok()
            .flatten()
            .or_else(|| window.primary_monitor().ok().flatten());

        let (mx, my, mw, mh) = if let Some(m) = monitor {
            let p = m.position();
            let s = m.size();
            (p.x, p.y, s.width as i32, s.height as i32)
        } else {
            (0, 0, 1920, 1080)
        };

        let (title, is_fullscreen) =
            win_drag::get_foreground_info(std::process::id(), mx, my, mw, mh);
        Ok(ActiveWindowContext {
            title,
            is_fullscreen,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Ok(ActiveWindowContext {
            title: String::new(),
            is_fullscreen: false,
        })
    }
}

#[tauri::command]
fn move_window_pos(window: tauri::WebviewWindow, x: i32, y: i32) -> Result<(), String> {
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_hit_rects(state: State<'_, HitState>, rects: Vec<HitRect>) {
    if let Ok(mut guard) = state.rects.lock() {
        *guard = rects;
    }
}

fn toggle_window_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(true) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn open_chat_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("aster-open-chat", ());
    }
}

fn open_settings_window_inner(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Aster • Configurações")
        .inner_size(760.0, 600.0)
        .resizable(true)
        .min_inner_size(700.0, 540.0)
        .additional_browser_args("--autoplay-policy=no-user-gesture-required")
        .center()
        .build();
}

#[tauri::command]
async fn open_settings(app: tauri::AppHandle) {
    open_settings_window_inner(&app);
}

#[tauri::command]
fn notify_memories_updated(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("aster-memories-updated", ());
    }
}

#[tauri::command]
fn reminders_send_action(app: tauri::AppHandle, payload: serde_json::Value) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("aster-reminder-action", payload);
    }
}

#[tauri::command]
fn reminders_broadcast_list(app: tauri::AppHandle, list: serde_json::Value) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.emit("aster-reminders-list", list);
    }
}

#[cfg(windows)]
fn spawn_global_hotkey(app_handle: tauri::AppHandle) {
    #[repr(C)]
    struct Msg {
        hwnd: *mut std::ffi::c_void,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterHotKey(h_wnd: *mut std::ffi::c_void, id: i32, fs_modifiers: u32, vk: u32) -> i32;
        fn GetMessageW(lp_msg: *mut Msg, h_wnd: *mut std::ffi::c_void, min: u32, max: u32) -> i32;
    }

    const MOD_ALT: u32 = 0x0001;
    const MOD_NOREPEAT: u32 = 0x4000;
    const VK_A: u32 = 0x41;
    const WM_HOTKEY: u32 = 0x0312;

    std::thread::spawn(move || unsafe {
        if RegisterHotKey(std::ptr::null_mut(), 1, MOD_ALT | MOD_NOREPEAT, VK_A) == 0 {
            return;
        }
        let mut msg = std::mem::zeroed::<Msg>();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wparam == 1 {
                open_chat_window(&app_handle);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let hit_rects: Arc<Mutex<Vec<HitRect>>> = Arc::new(Mutex::new(Vec::new()));
    let rects_for_thread = Arc::clone(&hit_rects);
    let voice_manager = Arc::new(voice::VoiceManager::new());
    let voice_for_exit = Arc::clone(&voice_manager);
    let always_on_top_flag = Arc::new(AtomicBool::new(true));
    let chatter_flag = Arc::new(AtomicBool::new(true));

    let app = tauri::Builder::default()
        .manage(HitState { rects: hit_rects })
        .manage(voice_manager)
        .setup(move |app| {
            // Atalho global Alt+A para chamar o chat da Aster de qualquer tela
            #[cfg(windows)]
            spawn_global_hotkey(app.handle().clone());

            // System Tray (bandeja do Windows)
            let autostart_initial = is_autostart_enabled();
            let autostart_label = if autostart_initial {
                "Iniciar com o Windows: ON"
            } else {
                "Iniciar com o Windows: OFF"
            };
            let chat_i = MenuItem::with_id(app, "chat", "Abrir Chat (Alt+A)", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Configurações & Memória", true, None::<&str>)?;
            let chatter_i = MenuItem::with_id(app, "chatter", "Falas Espontâneas: ON", true, None::<&str>)?;
            let roam_i = MenuItem::with_id(app, "roam", "Alternar Passeio Automático", true, None::<&str>)?;
            let autostart_i = MenuItem::with_id(app, "autostart", autostart_label, true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", "Mostrar / Esconder Aster", true, None::<&str>)?;
            let top_i = MenuItem::with_id(app, "top", "Alternar Sempre no Topo", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&chat_i, &settings_i, &chatter_i, &roam_i, &autostart_i, &toggle_i, &top_i, &quit_i],
            )?;

            let top_flag_menu = Arc::clone(&always_on_top_flag);
            let chatter_flag_menu = Arc::clone(&chatter_flag);
            let chatter_item_clone = chatter_i.clone();
            let autostart_item_clone = autostart_i.clone();
            let mut tray_builder = TrayIconBuilder::new()
                .tooltip("Aster Companion (Alt+A)")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "chat" => open_chat_window(app),
                    "settings" => open_settings_window_inner(app),
                    "chatter" => {
                        let next = !chatter_flag_menu.load(Ordering::Relaxed);
                        chatter_flag_menu.store(next, Ordering::Relaxed);
                        let _ = chatter_item_clone.set_text(if next {
                            "Falas Espontâneas: ON"
                        } else {
                            "Falas Espontâneas: OFF"
                        });
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.emit("aster-toggle-chatter", next);
                        }
                    }
                    "roam" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.emit("aster-toggle-roam", ());
                        }
                    }
                    "autostart" => {
                        let next = !is_autostart_enabled();
                        if set_autostart_enabled(next).is_ok() {
                            let _ = autostart_item_clone.set_text(if next {
                                "Iniciar com o Windows: ON"
                            } else {
                                "Iniciar com o Windows: OFF"
                            });
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.emit("aster-autostart-changed", next);
                            }
                        }
                    }
                    "toggle" => toggle_window_visibility(app),
                    "top" => {
                        let next = !top_flag_menu.load(Ordering::Relaxed);
                        top_flag_menu.store(next, Ordering::Relaxed);
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.set_always_on_top(next);
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window_visibility(tray.app_handle());
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            tray_builder.build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                // Posiciona no canto inferior direito respeitando a barra de tarefas
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let work_area = monitor.work_area();
                    let scale = monitor.scale_factor();
                    let win_w = (340.0 * scale) as i32;
                    let win_h = (580.0 * scale) as i32;
                    let margin_x = (20.0 * scale) as i32;
                    let margin_y = (10.0 * scale) as i32;

                    let x = work_area.position.x + work_area.size.width as i32 - win_w - margin_x;
                    let y = work_area.position.y + work_area.size.height as i32 - win_h - margin_y;

                    let _ = window.set_position(PhysicalPosition::new(x, y));
                }

                let _ = window.set_always_on_top(true);

                // Click-through inteligente: áreas transparentes passam cliques pro desktop
                let win_clone = window.clone();
                let rects_clone = Arc::clone(&rects_for_thread);
                std::thread::spawn(move || {
                    let mut currently_ignoring = false;
                    let pad = 6.0;

                    loop {
                        std::thread::sleep(Duration::from_millis(35));

                        if !win_clone.is_visible().unwrap_or(true) {
                            continue;
                        }

                        let Ok(cursor) = win_clone.cursor_position() else {
                            continue;
                        };
                        let Ok(win_pos) = win_clone.inner_position() else {
                            continue;
                        };
                        let Ok(scale) = win_clone.scale_factor() else {
                            continue;
                        };

                        let rel_x = (cursor.x - win_pos.x as f64) / scale;
                        let rel_y = (cursor.y - win_pos.y as f64) / scale;

                        let want_ignore = match rects_clone.lock() {
                            Ok(rects) if !rects.is_empty() => {
                                let over_ui = rects.iter().any(|r| {
                                    rel_x >= (r.x - pad)
                                        && rel_x <= (r.x + r.width + pad)
                                        && rel_y >= (r.y - pad)
                                        && rel_y <= (r.y + r.height + pad)
                                });
                                !over_ui
                            }
                            _ => false,
                        };

                        if want_ignore != currently_ignoring {
                            if win_clone.set_ignore_cursor_events(want_ignore).is_ok() {
                                currently_ignoring = want_ignore;
                            }
                        }
                    }
                });
            }

            // Garante Ollama rodando em segundo plano sem atrasar a abertura da Aster
            tauri::async_runtime::spawn(async {
                brain::ensure_ollama_running().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            update_hit_rects,
            get_desktop_bounds,
            move_window_pos,
            drag_start,
            drag_poll,
            find_window_perch,
            find_auto_perch,
            check_perch_alive,
            get_active_window_context,
            open_settings,
            notify_memories_updated,
            reminders_send_action,
            reminders_broadcast_list,
            config::get_settings,
            config::save_settings,
            docs::docs_list,
            docs::docs_open_folder,
            docs::docs_search,
            brain::chat_ollama,
            brain::chat_ollama_stream,
            brain::ask_ollama_vision,
            brain::list_ollama_models,
            brain::list_cloud_models,
            brain::test_ai_connection,
            brain::ask_hermes,
            brain::post_discord_webhook,
            brain::read_clipboard,
            brain::check_services_health,
            actions::open_app,
            actions::close_app,
            actions::open_url,
            actions::media_control,
            actions::play_spotify,
            actions::send_whatsapp,
            actions::send_discord_dm,
            voice::init_voice,
            voice::transcribe_audio,
            voice::synthesize_fish_speech,
            voice::stop_voice,
            memory::memory_list,
            memory::memory_add,
            memory::memory_remove,
            memory::memory_clear,
            skins::skins_list,
            skins::skins_get_overrides,
            skins::skins_set_active,
            skins::skins_create,
            skins::skins_import_folder,
            skins::skins_open_folder,
            media::get_now_playing,
            games::get_lol_state,
            games::detect_known_game,
        ])
        .build(tauri::generate_context!())
        .expect("erro ao construir Aster Companion");

    app.run(move |_app_handle, event| {
        if let RunEvent::Exit = event {
            // Encerra os workers Python limpinho sem deixar processos órfãos
            voice_for_exit.shutdown();
        }
    });
}
