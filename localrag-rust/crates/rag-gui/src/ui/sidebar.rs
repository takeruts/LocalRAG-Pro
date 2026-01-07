use eframe::egui;

use super::colors;
use crate::state::{AppState, Command};

/// サイドバーをレンダリング
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.add_space(16.0);

        // タイトル
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("LocalRAG Pro")
                    .size(24.0)
                    .color(colors::PRIMARY)
                    .strong(),
            );
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // Ollamaステータス
        render_ollama_status(ui, state);

        ui.add_space(16.0);

        // モデル選択
        render_model_selection(ui, state);

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        // インデックス管理
        render_indexing_section(ui, state);

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        // インデックス統計
        render_stats(ui, state);
    });
}

/// Ollamaステータス表示
fn render_ollama_status(ui: &mut egui::Ui, state: &AppState) {
    egui::Frame::none()
        .fill(colors::BG_CARD)
        .rounding(8.0)
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if state.ollama_running {
                    ui.label(
                        egui::RichText::new("●")
                            .size(14.0)
                            .color(colors::SUCCESS),
                    );
                    ui.label(
                        egui::RichText::new("Ollama 実行中")
                            .color(colors::SUCCESS)
                            .size(13.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("●")
                            .size(14.0)
                            .color(colors::ERROR),
                    );
                    ui.label(
                        egui::RichText::new("Ollama 停止中")
                            .color(colors::ERROR)
                            .size(13.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("↻").clicked() {
                        state.send_command(Command::RefreshModels);
                    }
                });
            });
        });
}

/// モデル選択UI
fn render_model_selection(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        egui::RichText::new("LLM モデル")
            .color(colors::TEXT_SECONDARY)
            .size(13.0),
    );
    ui.add_space(4.0);

    let _llm_response = egui::ComboBox::from_id_salt("llm_model")
        .width(ui.available_width() - 8.0)
        .selected_text(&state.llm_model)
        .show_ui(ui, |ui| {
            for model in &state.available_llm_models {
                if ui
                    .selectable_value(&mut state.llm_model, model.clone(), model)
                    .clicked()
                {
                    state.send_command(Command::SetLlmModel(model.clone()));
                }
            }
        });

    ui.add_space(12.0);
    ui.label(
        egui::RichText::new("Embedding モデル")
            .color(colors::TEXT_SECONDARY)
            .size(13.0),
    );
    ui.add_space(4.0);

    let _embed_response = egui::ComboBox::from_id_salt("embedding_model")
        .width(ui.available_width() - 8.0)
        .selected_text(&state.embedding_model)
        .show_ui(ui, |ui| {
            for model in &state.available_embedding_models {
                if ui
                    .selectable_value(&mut state.embedding_model, model.clone(), model)
                    .clicked()
                {
                    state.send_command(Command::SetEmbeddingModel(model.clone()));
                }
            }
        });
}

/// インデックス管理セクション
fn render_indexing_section(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        egui::RichText::new("ドキュメントフォルダ")
            .color(colors::TEXT_SECONDARY)
            .size(13.0),
    );
    ui.add_space(4.0);

    // フォルダパス表示
    egui::Frame::none()
        .fill(colors::BG_INPUT)
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            if let Some(path) = &state.folder_path {
                let full_path = path.display().to_string();
                ui.label(
                    egui::RichText::new(&full_path)
                        .color(colors::TEXT_PRIMARY)
                        .size(12.0),
                );
            } else {
                ui.label(
                    egui::RichText::new("フォルダを選択してください")
                        .color(colors::TEXT_MUTED)
                        .italics()
                        .size(12.0),
                );
            }
        });

    ui.add_space(10.0);

    // フォルダ選択ボタン
    if ui
        .add_sized(
            [ui.available_width(), 36.0],
            egui::Button::new(
                egui::RichText::new("フォルダを選択")
                    .size(14.0)
                    .color(colors::TEXT_BRIGHT),
            )
            .fill(colors::PRIMARY_DIM)
            .rounding(6.0),
        )
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            state.send_command(Command::SelectFolder(path));
        }
    }

    ui.add_space(10.0);

    // インデックス作成ボタン
    if !state.is_indexing {
        let enabled = state.folder_path.is_some() && state.ollama_running;
        if ui
            .add_enabled_ui(enabled, |ui| {
                ui.add_sized(
                    [ui.available_width(), 36.0],
                    egui::Button::new(
                        egui::RichText::new("インデックス作成")
                            .size(14.0)
                            .color(if enabled {
                                colors::TEXT_BRIGHT
                            } else {
                                colors::TEXT_MUTED
                            }),
                    )
                    .fill(if enabled {
                        colors::SUCCESS
                    } else {
                        colors::BG_CARD
                    })
                    .rounding(6.0),
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

        ui.add_space(6.0);

        if ui
            .add_sized(
                [ui.available_width(), 32.0],
                egui::Button::new(
                    egui::RichText::new("停止")
                        .size(13.0)
                        .color(colors::TEXT_BRIGHT),
                )
                .fill(colors::ERROR)
                .rounding(6.0),
            )
            .clicked()
        {
            state.send_command(Command::StopIndexing);
        }
    }
}

/// 統計情報表示
fn render_stats(ui: &mut egui::Ui, state: &AppState) {
    ui.label(
        egui::RichText::new("インデックス統計")
            .color(colors::TEXT_SECONDARY)
            .size(13.0),
    );
    ui.add_space(4.0);

    egui::Frame::none()
        .fill(colors::BG_CARD)
        .rounding(8.0)
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // インデックス済みフォルダを表示
                if let Some(ref folder) = state.index_stats.indexed_folder {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Folder")
                                .color(colors::TEXT_SECONDARY)
                                .size(12.0),
                        );
                    });
                    // フォルダパスを短縮表示
                    let display_path = shorten_path(folder, 30);
                    ui.label(
                        egui::RichText::new(&display_path)
                            .color(colors::PRIMARY)
                            .size(11.0),
                    );
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                }

                stat_row(ui, "Total Files", state.index_stats.total_files);
                ui.add_space(2.0);
                stat_row(ui, "Indexed", state.index_stats.indexed_files);
                if state.index_stats.skipped_files > 0 {
                    ui.add_space(2.0);
                    stat_row(ui, "Skipped", state.index_stats.skipped_files);
                }
                ui.add_space(2.0);
                stat_row(ui, "Chunks", state.index_stats.total_chunks);
                ui.add_space(2.0);
                stat_row(ui, "Embeddings", state.index_stats.total_embeddings);
            });
        });
}

/// パスを短縮表示する
fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // パスを分解
    let parts: Vec<&str> = path.split(['/', '\\']).collect();
    if parts.len() <= 2 {
        // 短すぎる場合は末尾を切り詰め
        return format!("{}...", &path[..max_len.saturating_sub(3)]);
    }

    // 最初と最後を残して中間を省略
    let first = parts.first().unwrap_or(&"");
    let last = parts.last().unwrap_or(&"");
    let second_last = parts.get(parts.len().saturating_sub(2)).unwrap_or(&"");

    format!("{}\\...\\{}\\{}", first, second_last, last)
}

/// 統計行
fn stat_row(ui: &mut egui::Ui, label: &str, value: usize) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(colors::TEXT_SECONDARY)
                .size(12.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value.to_string())
                    .color(colors::PRIMARY)
                    .size(13.0),
            );
        });
    });
}
