//! 統一されたカラーパレット
//! 全UIコンポーネントで共有

use eframe::egui::Color32;

// ========================================
// ベースカラー - ダークブルー系で統一
// ========================================

/// 背景色 - 最も暗い
pub const BG_DARK: Color32 = Color32::from_rgb(25, 28, 38);
/// カード背景 - やや明るい
pub const BG_CARD: Color32 = Color32::from_rgb(35, 40, 55);
/// カード背景（ホバー/アクティブ）
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(45, 52, 70);
/// 入力フィールド背景
pub const BG_INPUT: Color32 = Color32::from_rgb(40, 45, 60);

// ========================================
// アクセントカラー - ブルー系で統一
// ========================================

/// プライマリ - メインアクセント（明るいブルー）
pub const PRIMARY: Color32 = Color32::from_rgb(80, 160, 255);
/// プライマリ（暗め）- ボタン背景など
pub const PRIMARY_DIM: Color32 = Color32::from_rgb(60, 120, 200);
/// セカンダリ - 補助アクセント（シアン）
pub const SECONDARY: Color32 = Color32::from_rgb(80, 200, 220);

// ========================================
// テキストカラー
// ========================================

/// テキスト - 最も明るい（見出し、重要）
pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(250, 252, 255);
/// テキスト - 通常
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 225, 235);
/// テキスト - 補助（ラベルなど）
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 170, 190);
/// テキスト - 薄い（プレースホルダーなど）
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 110, 130);

// ========================================
// ステータスカラー
// ========================================

/// 成功 - グリーン
pub const SUCCESS: Color32 = Color32::from_rgb(80, 200, 120);
/// エラー - レッド
pub const ERROR: Color32 = Color32::from_rgb(255, 100, 100);
/// 警告/進行中 - オレンジ
pub const WARNING: Color32 = Color32::from_rgb(255, 180, 80);

// ========================================
// メッセージバブル
// ========================================

/// ユーザーメッセージ背景
pub const USER_MSG_BG: Color32 = Color32::from_rgb(60, 100, 180);
/// アシスタントメッセージ背景
pub const ASSISTANT_MSG_BG: Color32 = Color32::from_rgb(45, 52, 70);
