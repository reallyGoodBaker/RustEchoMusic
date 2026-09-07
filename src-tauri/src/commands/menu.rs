use tauri::{command, menu::MenuBuilder, Manager, Window};

#[command]
pub fn show_context_menu(window: Window) {
    let handle = window.app_handle();
    let menu = MenuBuilder::new(handle)
        .text("quit", "退出程序")
        .separator()
        .text("play", "播放")
        .build()
        .unwrap();
    let _ = window.popup_menu(&menu);
}
