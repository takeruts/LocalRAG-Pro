use eframe::egui;

use crate::state::{AppState, ChatMessage, Command, MessageRole};

/// チャットUIをレンダリング
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        // メッセージエリア
        let available_height = ui.available_height() - 120.0;

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(available_height)
            .show(ui, |ui| {
                for msg in &state.messages {
                    render_message(ui, msg);
                    ui.add_space(12.0);
                }
            });

        ui.add_space(8.0);

        // エージェント進捗表示
        if state.agent_mode && !state.agent_progress.is_empty() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(48, 48, 48))
                .rounding(8.0)
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&state.agent_progress)
                            .color(egui::Color32::from_rgb(255, 193, 7)),
                    );
                });
            ui.add_space(8.0);
        }

        // ソース表示
        if !state.current_sources.is_empty() && state.show_sources {
            render_sources(ui, state);
            ui.add_space(8.0);
        }

        ui.separator();
        ui.add_space(8.0);

        // 入力エリア
        render_input_area(ui, state);
    });
}

/// メッセージをレンダリング
fn render_message(ui: &mut egui::Ui, msg: &ChatMessage) {
    match msg.role {
        MessageRole::User => {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(33, 150, 243))
                        .rounding(12.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&msg.content)
                                    .color(egui::Color32::WHITE)
                                    .size(14.0),
                            );
                        });
                });
            });
        }
        MessageRole::Assistant => {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(48, 48, 48))
                .rounding(12.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&msg.content)
                            .color(egui::Color32::from_rgb(224, 224, 224))
                            .size(14.0),
                    );
                });
        }
    }
}

/// ソース情報表示
fn render_sources(ui: &mut egui::Ui, state: &AppState) {
    egui::CollapsingHeader::new(
        egui::RichText::new("📚 ソース情報")
            .size(13.0)
            .color(egui::Color32::from_rgb(156, 156, 156)),
    )
    .default_open(false)
    .show(ui, |ui| {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(36, 36, 36))
            .rounding(8.0)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                for (i, source) in state.current_sources.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}.", i + 1))
                                .color(egui::Color32::GRAY)
                                .size(12.0),
                        );

                        ui.vertical(|ui| {
                            let source_text = if let Some(page) = source.page {
                                format!("{} (P.{})", source.source, page + 1)
                            } else {
                                source.source.clone()
                            };

                            ui.label(
                                egui::RichText::new(source_text)
                                    .color(egui::Color32::from_rgb(224, 224, 224))
                                    .size(12.0),
                            );

                            ui.label(
                                egui::RichText::new(format!("Score: {:.2}", source.score))
                                    .color(egui::Color32::GRAY)
                                    .size(11.0),
                            );
                        });
                    });

                    if i < state.current_sources.len() - 1 {
                        ui.separator();
                    }
                }
            });
    });
}

/// 入力エリア
fn render_input_area(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        // エージェントモードトグル
        ui.toggle_value(&mut state.agent_mode, "🤖");

        if state.agent_mode {
            ui.label(
                egui::RichText::new("Agent")
                    .color(egui::Color32::from_rgb(255, 193, 7))
                    .size(12.0),
            );
        }

        ui.add_space(8.0);

        // テキスト入力
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.input_text)
                .hint_text("💬 Ask anything...")
                .desired_width(ui.available_width() - 60.0),
        );

        // Enter押下で送信
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            send_message(state);
        }

        // 送信ボタン
        let enabled = !state.input_text.trim().is_empty() && !state.is_generating;
        if ui
            .add_enabled(enabled, egui::Button::new("→").min_size([40.0, 24.0].into()))
            .clicked()
        {
            send_message(state);
        }
    });
}

/// メッセージ送信
fn send_message(state: &mut AppState) {
    let question = std::mem::take(&mut state.input_text);

    if question.trim().is_empty() {
        return;
    }

    // ユーザーメッセージ追加
    state.messages.push(ChatMessage::user(&question));

    // 空のアシスタントメッセージを追加（ストリーミング用）
    state.messages.push(ChatMessage::assistant(""));

    state.is_generating = true;
    state.current_sources.clear();
    state.agent_progress.clear();

    // クエリ送信
    if state.agent_mode {
        state.send_command(Command::SendAgentQuery(question));
    } else {
        state.send_command(Command::SendQuery(question));
    }
}
