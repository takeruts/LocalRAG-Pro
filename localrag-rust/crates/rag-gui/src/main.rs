mod app;
mod backend;
mod state;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    // ロギング初期化
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting LocalRAG Pro GUI");

    // ネイティブオプション設定
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // アプリ起動
    eframe::run_native(
        "LocalRAG Pro",
        native_options,
        Box::new(|cc| Ok(Box::new(app::RagApp::new(cc)))),
    )
}

/// アイコンをロード（オプション）
fn load_icon() -> egui::IconData {
    // デフォルトアイコン（32x32の透明PNG）
    let icon_bytes = vec![0u8; 32 * 32 * 4];

    egui::IconData {
        rgba: icon_bytes,
        width: 32,
        height: 32,
    }
}
