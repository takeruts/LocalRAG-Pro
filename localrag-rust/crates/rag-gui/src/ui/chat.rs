use eframe::egui;

use super::colors;
use crate::state::{AppState, ChatMessage, Command, MessageRole};

/// チャットUIをレンダリング
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // 生成中は継続的に再描画をリクエスト（スクロール追従のため）
    if state.is_generating {
        ui.ctx().request_repaint();
    }

    ui.vertical(|ui| {
        // メッセージエリア（入力エリアの高さを確保）
        let available_height = ui.available_height() - 80.0;

        // メッセージ数とソース数をIDに含めることで、変化時に自動スクロール
        let scroll_id = egui::Id::new("chat_scroll")
            .with(state.messages.len())
            .with(state.current_sources.len());

        egui::ScrollArea::vertical()
            .id_salt(scroll_id)
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .max_height(available_height)
            .show(ui, |ui| {
                ui.add_space(8.0);
                for (i, msg) in state.messages.iter().enumerate() {
                    let is_last = i == state.messages.len() - 1;
                    let is_generating_this = is_last && state.is_generating && msg.role == MessageRole::Assistant;
                    render_message(ui, msg, is_generating_this, state.agent_mode);
                    ui.add_space(12.0);
                }

                // エージェント進捗表示（スクロールエリア内）
                if state.agent_mode && !state.agent_progress.is_empty() {
                    render_agent_progress(ui, &state.agent_progress);
                    ui.add_space(8.0);
                }

                // 通常モードの思考中表示（スクロールエリア内）
                if !state.agent_mode && state.is_generating {
                    render_thinking_indicator(ui);
                    ui.add_space(8.0);
                }

                // ソース表示（スクロールエリア内）
                if !state.current_sources.is_empty() && state.show_sources {
                    render_sources(ui, state);
                    ui.add_space(8.0);
                }

                // 最下部にスペースを追加してスクロール余裕を確保
                ui.add_space(16.0);
            });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 入力エリア
        render_input_area(ui, state);
    });
}

/// 思考中インジケーター（通常RAGモード）
fn render_thinking_indicator(ui: &mut egui::Ui) {
    let time = ui.ctx().input(|i| i.time);
    let dots = match ((time * 2.0) as i32) % 4 {
        0 => ".",
        1 => "..",
        2 => "...",
        _ => "",
    };

    egui::Frame::none()
        .fill(colors::BG_CARD)
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // パルスアニメーション
                let pulse = ((time * 3.0).sin() * 0.5 + 0.5) as f32;
                let alpha = (150.0 + pulse * 105.0) as u8;

                ui.label(
                    egui::RichText::new(format!("検索中{}", dots))
                        .color(egui::Color32::from_rgba_unmultiplied(80, 160, 255, alpha))
                        .size(13.0),
                );
            });
        });
}

/// エージェント進捗表示（動的アニメーション付き）
fn render_agent_progress(ui: &mut egui::Ui, progress: &str) {
    let time = ui.ctx().input(|i| i.time);

    egui::Frame::none()
        .fill(colors::BG_CARD)
        .rounding(6.0)
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // 回転するスピナー風アニメーション
                let spinner_chars = ['◐', '◓', '◑', '◒'];
                let spinner_idx = ((time * 4.0) as usize) % spinner_chars.len();
                let spinner = spinner_chars[spinner_idx];

                // パルスアニメーション
                let pulse = ((time * 2.5).sin() * 0.5 + 0.5) as f32;
                let alpha = (180.0 + pulse * 75.0) as u8;

                ui.label(
                    egui::RichText::new(spinner.to_string())
                        .color(egui::Color32::from_rgba_unmultiplied(255, 180, 80, alpha))
                        .size(16.0),
                );

                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new(progress)
                        .color(colors::WARNING)
                        .size(13.0),
                );
            });
        });
}

/// メッセージをレンダリング
fn render_message(ui: &mut egui::Ui, msg: &ChatMessage, is_generating: bool, is_agent_mode: bool) {
    match msg.role {
        MessageRole::User => {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::Frame::none()
                        .fill(colors::USER_MSG_BG)
                        .rounding(12.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&msg.content)
                                    .color(colors::TEXT_BRIGHT)
                                    .size(14.0),
                            );
                        });
                });
            });
        }
        MessageRole::Assistant => {
            egui::Frame::none()
                .fill(colors::ASSISTANT_MSG_BG)
                .rounding(12.0)
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    if msg.content.is_empty() && is_generating {
                        // 生成中の空メッセージ - タイピングインジケーター
                        render_typing_indicator(ui, is_agent_mode);
                    } else {
                        ui.label(
                            egui::RichText::new(&msg.content)
                                .color(colors::TEXT_PRIMARY)
                                .size(14.0),
                        );

                        // 生成中なら末尾にカーソル
                        if is_generating && !msg.content.is_empty() {
                            let time = ui.ctx().input(|i| i.time);
                            let visible = ((time * 2.0) as i32) % 2 == 0;
                            if visible {
                                ui.label(
                                    egui::RichText::new("▌")
                                        .color(colors::PRIMARY)
                                        .size(14.0),
                                );
                            }
                        }
                    }
                });
        }
    }
}

/// タイピングインジケーター
fn render_typing_indicator(ui: &mut egui::Ui, is_agent_mode: bool) {
    let time = ui.ctx().input(|i| i.time);

    ui.horizontal(|ui| {
        // 3つのドットがウェーブアニメーション
        for i in 0..3 {
            let offset = i as f64 * 0.3;
            let bounce = ((time * 3.0 + offset).sin() * 0.5 + 0.5) as f32;
            let y_offset = bounce * 4.0;

            let color = if is_agent_mode {
                colors::WARNING
            } else {
                colors::PRIMARY
            };

            // ドットの透明度もアニメーション
            let alpha = (150.0 + bounce * 105.0) as u8;
            let animated_color = egui::Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                alpha,
            );

            let dot_rect = ui.available_rect_before_wrap();
            let center = egui::pos2(
                dot_rect.left() + 8.0 + (i as f32 * 12.0),
                dot_rect.top() + 8.0 - y_offset,
            );

            ui.painter().circle_filled(center, 4.0, animated_color);
        }

        ui.add_space(40.0);
    });
}

/// ソース情報表示
fn render_sources(ui: &mut egui::Ui, state: &AppState) {
    egui::CollapsingHeader::new(
        egui::RichText::new("ソース情報")
            .size(13.0)
            .color(colors::TEXT_SECONDARY),
    )
    .default_open(false)
    .show(ui, |ui| {
        egui::Frame::none()
            .fill(colors::BG_CARD)
            .rounding(6.0)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                for (i, source) in state.current_sources.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}.", i + 1))
                                .color(colors::PRIMARY)
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
                                    .color(colors::TEXT_PRIMARY)
                                    .size(12.0),
                            );

                            ui.label(
                                egui::RichText::new(format!("Score: {:.2}", source.score))
                                    .color(colors::TEXT_MUTED)
                                    .size(11.0),
                            );
                        });
                    });

                    if i < state.current_sources.len() - 1 {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);
                    }
                }
            });
    });
}

/// 入力エリア
fn render_input_area(ui: &mut egui::Ui, state: &mut AppState) {
    // モード切替の説明ラベル
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("モード:")
                .size(11.0)
                .color(colors::TEXT_MUTED),
        );

        ui.label(
            egui::RichText::new(if state.agent_mode {
                "Agent (自律検索)"
            } else {
                "RAG (単純検索)"
            })
            .size(11.0)
            .color(if state.agent_mode {
                colors::WARNING
            } else {
                colors::PRIMARY
            }),
        );

        ui.label(
            egui::RichText::new("← クリックで切替")
                .size(10.0)
                .color(colors::TEXT_MUTED),
        );
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        // エージェントモードトグル - 目立つデザイン
        let time = ui.ctx().input(|i| i.time);

        let (btn_color, text_color) = if state.agent_mode {
            // エージェントモード: オレンジ系でパルス
            let pulse = ((time * 2.0).sin() * 0.2 + 0.8) as f32;
            let r = (255.0 * pulse) as u8;
            let g = (150.0 * pulse) as u8;
            (
                egui::Color32::from_rgb(r, g, 50),
                colors::TEXT_BRIGHT,
            )
        } else {
            // RAGモード: ブルー系
            (
                colors::PRIMARY_DIM,
                colors::TEXT_BRIGHT,
            )
        };

        let agent_btn = ui.add(
            egui::Button::new(
                egui::RichText::new(if state.agent_mode { "Agent" } else { "RAG" })
                    .size(13.0)
                    .strong()
                    .color(text_color),
            )
            .fill(btn_color)
            .rounding(6.0)
            .min_size([60.0, 32.0].into()),
        );

        // ツールチップ
        if agent_btn.hovered() {
            egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("mode_tooltip"), |ui| {
                ui.label("クリックでモード切替");
                ui.label(if state.agent_mode {
                    "現在: Agent - 複数キーワードで自律検索"
                } else {
                    "現在: RAG - 質問をそのまま検索"
                });
            });
        }

        if agent_btn.clicked() {
            state.agent_mode = !state.agent_mode;
        }

        ui.add_space(8.0);

        // テキスト入力
        // 注: Enterキーでは送信しない（日本語IME変換との競合を避けるため）
        // 送信は右の送信ボタンをクリックして行う
        let _response = ui.add(
            egui::TextEdit::singleline(&mut state.input_text)
                .hint_text("質問を入力...")
                .desired_width(ui.available_width() - 70.0),
        );

        // 送信ボタン（インデックス中でも送信可能）
        let enabled = !state.input_text.trim().is_empty() && !state.is_generating;
        let send_btn = ui.add_enabled(
            enabled,
            egui::Button::new(
                egui::RichText::new("送信")
                    .size(13.0)
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
            .rounding(6.0)
            .min_size([50.0, 32.0].into()),
        );

        if send_btn.clicked() {
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
