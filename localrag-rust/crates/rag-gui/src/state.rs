use std::path::PathBuf;
use tokio::sync::mpsc;

/// メッセージの種類
#[derive(Clone, Debug, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
}

/// チャットメッセージ
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// インデックス統計情報
#[derive(Clone, Debug, Default)]
pub struct IndexStats {
    pub total_files: usize,
    pub indexed_files: usize,
    pub skipped_files: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub indexed_folder: Option<String>,  // インデックスされたフォルダのパス
}

/// ソース情報
#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub source: String,
    pub page: Option<usize>,
    pub score: f32,
}

/// バックエンドコマンド
pub enum Command {
    SelectFolder(PathBuf),
    StartIndexing,
    StopIndexing,
    SendQuery(String),
    SendAgentQuery(String),
    RefreshModels,
    SetLlmModel(String),
    SetEmbeddingModel(String),
}

/// バックエンドイベント
pub enum Event {
    FolderSelected(PathBuf),
    IndexProgress { progress: f32, file: String },
    IndexStatsUpdate { stats: IndexStats },  // リアルタイム統計更新
    IndexComplete { stats: IndexStats },
    QueryChunk(String),
    QueryComplete { sources: Vec<SourceInfo> },
    AgentProgress(String),
    OllamaStatus(bool),
    ModelsRefreshed { llm_models: Vec<String>, embedding_models: Vec<String> },
    Error(String),
}

/// アプリケーション状態
pub struct AppState {
    // モデル設定
    pub llm_model: String,
    pub embedding_model: String,
    pub available_llm_models: Vec<String>,
    pub available_embedding_models: Vec<String>,

    // Ollama状態
    pub ollama_running: bool,

    // インデックス
    pub folder_path: Option<PathBuf>,
    pub is_indexing: bool,
    pub index_progress: f32,
    pub current_file: String,
    pub index_stats: IndexStats,

    // チャット
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    pub is_generating: bool,
    pub agent_mode: bool,
    pub agent_progress: String,

    // ソース表示
    pub current_sources: Vec<SourceInfo>,
    pub show_sources: bool,

    // バックエンド通信
    pub command_tx: mpsc::Sender<Command>,
    pub event_rx: mpsc::Receiver<Event>,
}

impl AppState {
    pub fn new(command_tx: mpsc::Sender<Command>, event_rx: mpsc::Receiver<Event>) -> Self {
        Self {
            llm_model: "gemma3:4b".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            available_llm_models: vec![],
            available_embedding_models: vec![],
            ollama_running: false,
            folder_path: None,
            is_indexing: false,
            index_progress: 0.0,
            current_file: String::new(),
            index_stats: IndexStats::default(),
            messages: vec![],
            input_text: String::new(),
            is_generating: false,
            agent_mode: false,
            agent_progress: String::new(),
            current_sources: vec![],
            show_sources: true,
            command_tx,
            event_rx,
        }
    }

    /// イベントを処理
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::FolderSelected(path) => {
                self.folder_path = Some(path);
            }
            Event::IndexProgress { progress, file } => {
                self.is_indexing = true;
                self.index_progress = progress;
                self.current_file = file;
            }
            Event::IndexStatsUpdate { stats } => {
                // リアルタイム統計更新（インデックス中は進行状態を維持）
                self.index_stats = stats;
            }
            Event::IndexComplete { stats } => {
                self.is_indexing = false;
                self.index_progress = 1.0;
                self.index_stats = stats;
            }
            Event::QueryChunk(chunk) => {
                if let Some(last_msg) = self.messages.last_mut() {
                    if matches!(last_msg.role, MessageRole::Assistant) {
                        last_msg.content.push_str(&chunk);
                    }
                } else {
                    self.messages.push(ChatMessage::assistant(chunk));
                }
            }
            Event::QueryComplete { sources } => {
                self.is_generating = false;
                self.current_sources = sources;
            }
            Event::AgentProgress(progress) => {
                self.agent_progress = progress;
            }
            Event::OllamaStatus(running) => {
                self.ollama_running = running;
            }
            Event::ModelsRefreshed { llm_models, embedding_models } => {
                self.available_llm_models = llm_models;
                self.available_embedding_models = embedding_models;
            }
            Event::Error(err) => {
                self.messages.push(ChatMessage::assistant(format!("❌ エラー: {}", err)));
                self.is_generating = false;
                self.is_indexing = false;
            }
        }
    }

    /// コマンドを送信
    pub fn send_command(&self, cmd: Command) {
        let _ = self.command_tx.try_send(cmd);
    }
}
