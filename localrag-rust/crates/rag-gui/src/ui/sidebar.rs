use eframe::egui;

use crate::state::{AppState, Command};

/// サイドバーをレンダリング
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.add_space(12.0);

        // タイトル
        ui.heading(
            egui::RichText::new("⚡ LocalRAG Pro")
                .size(24.0)
                .color(egui::Color32::from_rgb(79, 195, 247)),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Ollamaステータス
        render_ollama_status(ui, state);

        ui.add_space(12.0);

        // モデル選択
        render_model_selection(ui, state);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // インデックス管理
        render_indexing_section(ui, state);

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // インデックス統計
        render_stats(ui, state);
    });
}

/// Ollamaステータス表示
fn render_ollama_status(ui: &mut egui::Ui, state: &AppState) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(36, 36, 36))
        .rounding(8.0)
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if state.ollama_running {
                    ui.label(
                        egui::RichText::new("●")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(76, 175, 80)),
                    );
                    ui.label(
                        egui::RichText::new("Ollama 実行中")
                            .color(egui::Color32::from_rgb(76, 175, 80)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("●")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(244, 67, 54)),
                    );
                    ui.label(
                        egui::RichText::new("Ollama 停止中")
                            .color(egui::Color32::from_rgb(244, 67, 54)),
                    );
                }

                if ui.small_button("🔄").clicked() {
                    state.send_command(Command::RefreshModels);
                }
            });
        });
}

/// モデル選択UI
fn render_model_selection(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("💬 LLMモデル").strong());
    ui.add_space(4.0);

    let llm_response = egui::ComboBox::from_id_salt("llm_model")
        .width(ui.available_width() - 8.0)
        .selected_text(&state.llm_model)
        .show_ui(ui, |ui| {
            for model in &state.available_llm_models {
                if ui.selectable_value(&mut state.llm_model, model.clone(), model).clicked() {
                    state.send_command(Command::SetLlmModel(model.clone()));
                }
            }
        });

    ui.add_space(8.0);
    ui.label(egui::RichText::new("🔢 Embeddingモデル").strong());
    ui.add_space(4.0);

    let embed_response = egui::ComboBox::from_id_salt("embedding_model")
        .width(ui.available_width() - 8.0)
        .selected_text(&state.embedding_model)
        .show_ui(ui, |ui| {
            for model in &state.available_embedding_models {
                if ui.selectable_value(&mut state.embedding_model, model.clone(), model).clicked() {
                    state.send_command(Command::SetEmbeddingModel(model.clone()));
                }
            }
        });
}

/// インデックス管理セクション
fn render_indexing_section(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("📁 ドキュメントフォルダ").strong());
    ui.add_space(4.0);

    // フォルダパス表示
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(48, 48, 48))
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            if let Some(path) = &state.folder_path {
                // フルパスを表示
                let full_path = path.display().to_string();

                // パスを省略形で表示（長すぎる場合）
                let max_width = ui.available_width() - 20.0;
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let text_galley = ui.fonts(|f| f.layout_no_wrap(full_path.clone(), font_id.clone(), egui::Color32::WHITE));

                if text_galley.size().x > max_width {
                    // パスが長い場合は...で省略
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&full_path).color(egui::Color32::from_rgb(200, 200, 200)).small());
                    });
                } else {
                    ui.label(egui::RichText::new(&full_path).color(egui::Color32::from_rgb(200, 200, 200)));
                }
            } else {
                ui.label(
                    egui::RichText::new("フォルダを選択してください")
                        .color(egui::Color32::from_rgb(128, 128, 128))
                        .italics()
                );
            }
        });

    ui.add_space(8.0);

    // フォルダ選択ボタン
    if ui
        .add_sized(
            [ui.available_width(), 32.0],
            egui::Button::new("📂 フォルダを選択"),
        )
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            state.send_command(Command::SelectFolder(path));
        }
    }

    ui.add_space(8.0);

    // インデックス作成ボタン
    if !state.is_indexing {
        let enabled = state.folder_path.is_some() && state.ollama_running;
        if ui
            .add_enabled_ui(enabled, |ui| {
                ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::Button::new(egui::RichText::new("⚡ インデックス作成").size(14.0)),
                )
            })
            .inner
            .clicked()
        {
            state.send_command(Command::StartIndexing);
        }
    } else {
        // 進捗バー
        ui.add(
            egui::ProgressBar::new(state.index_progress)
                .text(&state.current_file)
                .animate(true),
        );

        ui.add_space(4.0);

        if ui
            .add_sized([ui.available_width(), 32.0], egui::Button::new("⏸ 停止"))
            .clicked()
        {
            state.send_command(Command::StopIndexing);
        }
    }
}

/// 統計情報表示
fn render_stats(ui: &mut egui::Ui, state: &AppState) {
    ui.label(egui::RichText::new("📊 インデックス統計").strong());
    ui.add_space(4.0);

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(36, 36, 36))
        .rounding(8.0)
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                stat_row(ui, "Total Files:", state.index_stats.total_files);
                stat_row(ui, "Indexed:", state.index_stats.indexed_files);
                stat_row(ui, "Chunks:", state.index_stats.total_chunks);
                stat_row(ui, "Embeddings:", state.index_stats.total_embeddings);
            });
        });
}

/// 統計行
fn stat_row(ui: &mut egui::Ui, label: &str, value: usize) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(egui::Color32::GRAY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value.to_string())
                    .color(egui::Color32::from_rgb(79, 195, 247)),
            );
        });
    });
}
