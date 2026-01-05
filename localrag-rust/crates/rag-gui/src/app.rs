use eframe::egui;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::backend::Backend;
use crate::state::{AppState, ChatMessage, Command, Event, MessageRole};
use crate::ui;

/// メインアプリケーション
pub struct RagApp {
    state: AppState,
    backend: Arc<Backend>,
    runtime: tokio::runtime::Runtime,
}

impl RagApp {
    /// 日本語フォントを設定
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Windowsシステムフォントを読み込み
        // MS Gothic, Yu Gothic, Meiryo など一般的な日本語フォントを試す
        let font_paths = vec![
            r"C:\Windows\Fonts\msgothic.ttc",  // MS Gothic
            r"C:\Windows\Fonts\YuGothR.ttc",   // Yu Gothic Regular
            r"C:\Windows\Fonts\YuGothM.ttc",   // Yu Gothic Medium
            r"C:\Windows\Fonts\meiryo.ttc",    // Meiryo
            r"C:\Windows\Fonts\msmincho.ttc",  // MS Mincho
        ];

        let mut font_loaded = false;
        for font_path in font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "japanese".to_owned(),
                    Arc::new(egui::FontData::from_owned(font_data)),
                );
                font_loaded = true;
                tracing::info!("Loaded Japanese font from: {}", font_path);
                break;
            }
        }

        if !font_loaded {
            tracing::warn!("No Japanese font found, text may appear garbled");
        } else {
            // 日本語フォントを全てのフォントファミリーに追加
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("japanese".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("japanese".to_owned());
        }

        ctx.set_fonts(fonts);
    }

    /// 新しいアプリを作成
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 日本語フォント設定
        Self::setup_fonts(&cc.egui_ctx);

        // デフォルトテーマ設定
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        cc.egui_ctx.set_style(style);

        // 非同期ランタイム
        let runtime = tokio::runtime::Runtime::new().unwrap();

        // チャネル作成
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);

        // バックエンド起動
        let backend = Arc::new(Backend::new(event_tx));
        let backend_clone = backend.clone();

        runtime.spawn(async move {
            backend_clone.run(command_rx).await;
        });

        // 状態初期化
        let state = AppState::new(command_tx, event_rx);

        // 初期化コマンド
        state.send_command(Command::RefreshModels);

        Self {
            state,
            backend,
            runtime,
        }
    }

    /// サイドバーをレンダリング
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui::sidebar::render(ui, &mut self.state);
    }

    /// チャットエリアをレンダリング
    fn render_chat(&mut self, ui: &mut egui::Ui) {
        ui::chat::render(ui, &mut self.state);
    }
}

impl eframe::App for RagApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // イベント処理
        while let Ok(event) = self.state.event_rx.try_recv() {
            self.state.handle_event(event);
        }

        // サイドバー
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(360.0)
            .min_width(300.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // チャットエリア
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_chat(ui);
        });

        // 定期再描画（ストリーミング更新用）
        if self.state.is_generating || self.state.is_indexing {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // クリーンアップ
        tracing::info!("Shutting down application");
    }
}
