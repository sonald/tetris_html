#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // // Set up window focus/blur event listeners
            // let main_window = app.get_webview_window("main").unwrap();

            // let main_window_clone = main_window.clone();
            // main_window.on_window_event(move |event| {
            //     match event {
            //         tauri::WindowEvent::Focused(focused) => {
            //             if focused {
            //                 // Window gained focus
            //                 let _ = main_window_clone.emit("tauri://window-focus", ());
            //             } else {
            //                 // Window lost focus
            //                 let _ = main_window_clone.emit("tauri://window-blur", ());
            //             }
            //         }
            //         _ => {}
            //     }
            // });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
