mod app;
mod backend;
mod state;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    // ロギング初期化（DEBUGレベルに変更 + ファイル出力）
    use std::fs::File;
    use std::path::PathBuf;
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    // 実行ファイルと同じディレクトリにログを作成
    let exe_path = std::env::current_exe().expect("Failed to get exe path");
    let exe_dir = exe_path.parent().expect("Failed to get exe directory");
    let log_path = exe_dir.join("localrag_debug.log");

    println!("Creating log file at: {:?}", log_path);
    let log_file = File::create(&log_path).expect("Failed to create log file");
    let (non_blocking, guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(true)
        .with_writer(std::io::stdout.and(non_blocking))
        .init();

    tracing::info!("======================================");
    tracing::info!("Starting LocalRAG Pro GUI");
    tracing::info!("Log file: {:?}", log_path);
    tracing::info!("======================================");

    // ネイティブオプション設定
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // アプリ起動
    let result = eframe::run_native(
        "LocalRAG Pro",
        native_options,
        Box::new(|cc| Ok(Box::new(app::RagApp::new(cc)))),
    );

    // guardを保持してログをフラッシュ
    drop(guard);

    result
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
