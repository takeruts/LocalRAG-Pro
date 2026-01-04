import customtkinter as ctk
import os
import sys
import threading
import ctypes
from tkinter import filedialog, messagebox

# --- 高DPIディスプレイ対応 ---
try:
    ctypes.windll.shcore.SetProcessDpiAwareness(1)
except (AttributeError, OSError):
    try:
        ctypes.windll.user32.SetProcessDPIAware()
    except (AttributeError, OSError):
        pass

# --- AI関連ライブラリ ---
from langchain_community.document_loaders import PyMuPDFLoader, TextLoader, Docx2txtLoader, UnstructuredExcelLoader
from langchain_core.documents import Document
from langchain_text_splitters import RecursiveCharacterTextSplitter
from langchain_community.vectorstores import Chroma
from langchain_community.embeddings import OllamaEmbeddings
from langchain_community.llms import Ollama
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser

# リランカー機能は削除（Ollama専用版）
HAS_RERANKER = False

# --- パス解決 & 環境変数 ---
frozen = getattr(sys, 'frozen', False)
current_dir = os.path.dirname(sys.executable) if frozen else os.path.dirname(os.path.abspath(__file__))
BASE_DB_DIR = os.path.join(current_dir, "chroma_db")
MODELS_DIR = os.path.join(current_dir, "models")
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

for d in [BASE_DB_DIR, MODELS_DIR]:
    os.makedirs(d, exist_ok=True)

# --- Ollama関連のヘルパー関数 ---
def check_ollama_running():
    """Ollamaが実行中かチェック"""
    try:
        import requests
        response = requests.get("http://localhost:11434/api/tags", timeout=2)
        return response.status_code == 200
    except:
        return False

def get_ollama_models():
    """利用可能なOllama LLMモデルのリストを取得"""
    try:
        import requests
        response = requests.get("http://localhost:11434/api/tags", timeout=2)
        if response.status_code == 200:
            data = response.json()
            all_models = [model['name'] for model in data.get('models', [])]
            # embeddingモデルを除外
            llm_models = [m for m in all_models if 'embed' not in m.lower()]
            return llm_models if llm_models else ["gemma2:2b", "gemma2:9b", "qwen2.5:7b"]
        return ["gemma2:2b", "gemma2:9b", "qwen2.5:7b"]
    except:
        return ["gemma2:2b", "gemma2:9b", "qwen2.5:7b"]

def get_ollama_embedding_models():
    """利用可能なOllama Embeddingモデルのリストを取得"""
    try:
        import requests
        response = requests.get("http://localhost:11434/api/tags", timeout=2)
        if response.status_code == 200:
            data = response.json()
            all_models = [model['name'] for model in data.get('models', [])]
            # embeddingモデルのみ抽出
            embed_models = [m for m in all_models if 'embed' in m.lower()]
            return embed_models if embed_models else ["nomic-embed-text", "mxbai-embed-large", "all-minilm"]
        return ["nomic-embed-text", "mxbai-embed-large", "all-minilm"]
    except:
        return ["nomic-embed-text", "mxbai-embed-large", "all-minilm"]

class RAGWinApp(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.stop_requested = False
        self.is_indexing = False
        self.folder_path = ""
        self.source_buttons = []
        self.error_files = []
        self.ollama_model = "gemma2:2b"  # デフォルトLLMモデル
        self.embedding_model = "nomic-embed-text"  # デフォルトEmbeddingモデル

        ctk.set_appearance_mode("dark")
        ctk.set_default_color_theme("blue")

        # モダンなフォント設定
        self.font_title = ctk.CTkFont(family="Segoe UI", size=24, weight="bold")
        self.font_section = ctk.CTkFont(family="Segoe UI", size=16, weight="bold")
        self.font_main = ctk.CTkFont(family="Segoe UI", size=13)
        self.font_bold = ctk.CTkFont(family="Segoe UI", size=14, weight="bold")
        self.font_mini = ctk.CTkFont(family="Segoe UI", size=11)
        self.font_chat = ctk.CTkFont(family="Segoe UI", size=13)

        self.title("LocalRAG Pro • Ollama Edition")
        self.geometry("1400x920")
        self.minsize(1200, 700)
        
        self.sidebar_width = 360
        self.grid_columnconfigure(0, minsize=self.sidebar_width)
        self.grid_columnconfigure(1, weight=0)
        self.grid_columnconfigure(2, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # --- サイドバー ---
        self.sidebar = ctk.CTkFrame(self, width=self.sidebar_width, corner_radius=0, fg_color="#1a1a1a")
        self.sidebar.grid(row=0, column=0, sticky="nsew")
        self.sidebar.grid_propagate(False)

        # タイトルセクション
        title_frame = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        title_frame.pack(pady=(25, 20), padx=20, fill="x")

        ctk.CTkLabel(
            title_frame,
            text="⚡",
            font=ctk.CTkFont(size=32)
        ).pack()

        ctk.CTkLabel(
            title_frame,
            text="LocalRAG Pro",
            font=self.font_title,
            text_color="#4fc3f7"
        ).pack()

        ctk.CTkLabel(
            title_frame,
            text="Powered by Ollama",
            font=self.font_mini,
            text_color="#64b5f6"
        ).pack()

        # Ollamaステータスカード
        self.ollama_status_frame = ctk.CTkFrame(
            self.sidebar,
            fg_color="#242424",
            corner_radius=12
        )
        self.ollama_status_frame.pack(pady=(0, 15), padx=20, fill="x")
        self.ollama_status_label = ctk.CTkLabel(
            self.ollama_status_frame,
            text="",
            font=self.font_mini
        )
        self.ollama_status_label.pack(pady=10)

        # モデル設定セクション
        models_section = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        models_section.pack(pady=0, padx=20, fill="x")

        ctk.CTkLabel(
            models_section,
            text="🤖 AI Models",
            font=self.font_section,
            text_color="#81c784",
            anchor="w"
        ).pack(pady=(0, 12), fill="x")

        # LLMモデル選択カード
        llm_card = ctk.CTkFrame(models_section, fg_color="#242424", corner_radius=10)
        llm_card.pack(pady=(0, 10), fill="x")

        ctk.CTkLabel(
            llm_card,
            text="💬 LLM",
            font=self.font_mini,
            text_color="#90caf9",
            anchor="w"
        ).pack(pady=(8, 4), padx=12, fill="x")

        self.ollama_models = get_ollama_models()
        self.ollama_model_option = ctk.CTkOptionMenu(
            llm_card,
            values=self.ollama_models if self.ollama_models else ["モデルなし"],
            font=self.font_main,
            command=self.on_model_change,
            fg_color="#1e88e5",
            button_color="#1565c0",
            button_hover_color="#0d47a1",
            dropdown_fg_color="#2a2a2a",
            corner_radius=8
        )
        if self.ollama_models:
            self.ollama_model = self.ollama_models[0]
            self.ollama_model_option.set(self.ollama_models[0])
        self.ollama_model_option.pack(pady=(0, 8), padx=12, fill="x")

        # Embeddingモデル選択カード
        embed_card = ctk.CTkFrame(models_section, fg_color="#242424", corner_radius=10)
        embed_card.pack(pady=(0, 0), fill="x")

        ctk.CTkLabel(
            embed_card,
            text="📊 Embedding",
            font=self.font_mini,
            text_color="#ce93d8",
            anchor="w"
        ).pack(pady=(8, 4), padx=12, fill="x")

        self.embedding_models = get_ollama_embedding_models()
        self.embedding_model_option = ctk.CTkOptionMenu(
            embed_card,
            values=self.embedding_models if self.embedding_models else ["モデルなし"],
            font=self.font_main,
            command=self.on_embedding_model_change,
            fg_color="#7b1fa2",
            button_color="#6a1b9a",
            button_hover_color="#4a148c",
            dropdown_fg_color="#2a2a2a",
            corner_radius=8
        )
        if self.embedding_models:
            self.embedding_model = self.embedding_models[0]
            self.embedding_model_option.set(self.embedding_models[0])
        self.embedding_model_option.pack(pady=(0, 8), padx=12, fill="x")

        # データ管理セクション
        data_section = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        data_section.pack(pady=(20, 0), padx=20, fill="x")

        ctk.CTkLabel(
            data_section,
            text="📁 Data Management",
            font=self.font_section,
            text_color="#ffb74d",
            anchor="w"
        ).pack(pady=(0, 12), fill="x")

        self.btn_folder = ctk.CTkButton(
            data_section,
            text="📁 Select Folder",
            font=self.font_main,
            corner_radius=10,
            command=self.select_folder,
            fg_color="#424242",
            hover_color="#616161",
            height=40
        )
        self.btn_folder.pack(pady=(0, 12), fill="x")

        self.agent_switch = ctk.CTkSwitch(
            data_section,
            text="🤖 Agent Mode (Autonomous)",
            font=self.font_main,
            progress_color="#66bb6a",
            button_color="#43a047",
            button_hover_color="#2e7d32"
        )
        self.agent_switch.pack(pady=(0, 15))

        # アクションボタングループ
        action_frame = ctk.CTkFrame(data_section, fg_color="transparent")
        action_frame.pack(fill="x")

        self.btn_scan = ctk.CTkButton(
            action_frame,
            text="⚡ Start Indexing",
            font=self.font_bold,
            fg_color="#4caf50",
            hover_color="#388e3c",
            command=self.start_scan,
            corner_radius=10,
            height=45
        )
        self.btn_scan.pack(pady=(0, 8), fill="x")

        self.btn_stop = ctk.CTkButton(
            action_frame,
            text="🛑 Stop",
            font=self.font_main,
            fg_color="#f44336",
            hover_color="#d32f2f",
            state="disabled",
            command=self.request_stop,
            corner_radius=10,
            height=38
        )
        self.btn_stop.pack(fill="x")

        # プログレスセクション
        progress_section = ctk.CTkFrame(self.sidebar, fg_color="#242424", corner_radius=12)
        progress_section.pack(pady=(20, 0), padx=20, fill="x")

        self.load_label = ctk.CTkLabel(
            progress_section,
            text="✨ Ready",
            font=self.font_mini,
            text_color="#81c784",
            anchor="w"
        )
        self.load_label.pack(pady=(12, 6), padx=15, anchor="w")

        self.p_bar = ctk.CTkProgressBar(
            progress_section,
            height=8,
            corner_radius=4,
            progress_color="#4fc3f7",
            fg_color="#424242"
        )
        self.p_bar.set(0)
        self.p_bar.pack(pady=(0, 8), padx=15, fill="x")

        self.file_name_label = ctk.CTkLabel(
            progress_section,
            text="",
            font=self.font_mini,
            text_color="#90caf9",
            wraplength=300,
            anchor="w",
            justify="left"
        )
        self.file_name_label.pack(pady=(0, 8), padx=15, anchor="w")

        self.error_label = ctk.CTkLabel(
            progress_section,
            text="",
            font=self.font_mini,
            text_color="#ef5350",
            wraplength=300,
            anchor="w",
            justify="left"
        )
        self.error_label.pack(pady=(0, 12), padx=15, anchor="w")

        self.db_label = ctk.CTkLabel(self.sidebar, text="DB登録: ---", font=self.font_mini, text_color="#FFB74D")
        self.db_label.pack(pady=(5, 0), padx=20, anchor="w")

        # 参照資料セクション
        sources_header = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        sources_header.pack(pady=(10, 5), padx=20, fill="x")

        ctk.CTkLabel(
            sources_header,
            text="📚 Sources",
            font=self.font_section,
            text_color="#ba68c8",
            anchor="w"
        ).pack(fill="x")

        self.source_frame = ctk.CTkScrollableFrame(
            self.sidebar,
            fg_color="#242424",
            corner_radius=10
        )
        self.source_frame.pack(fill="both", expand=True, padx=20, pady=(0, 15))

        # リサイザー
        self.resizer = ctk.CTkFrame(self, width=3, cursor="sb_h_double_arrow", fg_color="#2a2a2a")
        self.resizer.grid(row=0, column=1, sticky="ns")
        self.resizer.bind("<B1-Motion>", self.on_resize)

        # メインチャットエリア
        self.chat_frame = ctk.CTkFrame(self, fg_color="#0d0d0d", corner_radius=0)
        self.chat_frame.grid(row=0, column=2, padx=0, pady=0, sticky="nsew")
        self.chat_frame.grid_columnconfigure(0, weight=1)
        self.chat_frame.grid_rowconfigure(1, weight=1)  # チャット表示エリアを伸縮可能に

        # チャットヘッダー
        chat_header = ctk.CTkFrame(self.chat_frame, fg_color="#1a1a1a", height=60)
        chat_header.grid(row=0, column=0, sticky="ew", padx=0, pady=0)
        chat_header.grid_columnconfigure(0, weight=1)
        chat_header.grid_propagate(False)

        ctk.CTkLabel(
            chat_header,
            text="💬 Chat",
            font=self.font_section,
            text_color="#4fc3f7",
            anchor="w"
        ).pack(side="left", padx=20, pady=15)

        # チャット表示エリア
        self.chat_display = ctk.CTkTextbox(
            self.chat_frame,
            state="disabled",
            font=self.font_chat,
            fg_color="#1a1a1a",
            wrap="word",
            corner_radius=0,
            border_width=0
        )
        self.chat_display.grid(row=1, column=0, padx=20, pady=(10, 10), sticky="nsew")

        # 入力エリア
        self.input_area = ctk.CTkFrame(self.chat_frame, fg_color="#1a1a1a", corner_radius=0)
        self.input_area.grid(row=2, column=0, padx=20, pady=(0, 20), sticky="ew")
        self.input_area.grid_columnconfigure(0, weight=1)

        self.entry = ctk.CTkEntry(
            self.input_area,
            placeholder_text="💬 Ask anything...",
            height=55,
            corner_radius=28,
            font=self.font_main,
            fg_color="#242424",
            border_color="#424242",
            border_width=2
        )
        self.entry.grid(row=0, column=0, sticky="ew", padx=(0, 12))
        self.entry.bind("<Return>", lambda e: self.send_query())

        self.btn_send = ctk.CTkButton(
            self.input_area,
            text="→",
            width=55,
            height=55,
            command=self.send_query,
            corner_radius=28,
            font=ctk.CTkFont(size=24),
            fg_color="#1e88e5",
            hover_color="#1565c0"
        )
        self.btn_send.grid(row=0, column=1)

        # 起動時にOllamaステータスをチェック
        self.after(500, self.check_ollama_status)

    def get_model_config(self):
        """モデル設定を取得する（Ollama版）"""
        # 選択されたEmbeddingモデルを使用
        embed_model = self.embedding_model
        # モデル名からディレクトリ名を生成（特殊文字を除去）
        safe_name = embed_model.replace(':', '_').replace('-', '_').replace('.', '_')
        db_dir = os.path.join(BASE_DB_DIR, f"ollama_{safe_name}")
        return embed_model, db_dir

    def clean_metadata_value(self, v):
        """メタデータの値をクリーンにする"""
        if v is None:
            return ""
        if isinstance(v, (str, int, float, bool)):
            return v
        return str(v)

    def create_embeddings(self, model_name):
        """Embeddingモデルを作成する（Ollama使用）"""
        # 指定されたOllamaのembeddingモデルを使用
        return OllamaEmbeddings(
            model=model_name
        )

    def check_ollama_status(self):
        """Ollamaの状態をチェックしてUIを更新"""
        if check_ollama_running():
            self.ollama_status_label.configure(
                text="✓ Ollama 実行中",
                text_color="#4CAF50"
            )
            # モデルリストを更新
            models = get_ollama_models()
            if models != self.ollama_models:
                self.ollama_models = models
                self.ollama_model_option.configure(values=models)
                if models and self.ollama_model not in models:
                    self.ollama_model = models[0]
                    self.ollama_model_option.set(models[0])
        else:
            self.ollama_status_label.configure(
                text="⚠ Ollama 未起動 - インストール・起動してください",
                text_color="#FF9800"
            )
            messagebox.showwarning(
                "Ollama未起動",
                "Ollamaが実行されていません。\n\n"
                "インストール方法:\n"
                "1. https://ollama.com/download からダウンロード\n"
                "2. インストール後、コマンドプロンプトで:\n"
                "   ollama run gemma2:2b\n\n"
                "Ollamaを起動してから再度お試しください。"
            )
        # 5秒後に再チェック
        self.after(5000, self.check_ollama_status)

    def on_model_change(self, choice):
        """LLMモデル選択が変更されたときの処理"""
        self.ollama_model = choice
        self.update_chat("System", f"LLMモデルを変更: {choice}")

    def on_embedding_model_change(self, choice):
        """Embeddingモデル選択が変更されたときの処理"""
        self.embedding_model = choice
        self.update_chat("System", f"Embeddingモデルを変更: {choice}\n※次回のIndexing時に適用されます")

    def on_resize(self, event):
        new_width = event.x_root - self.winfo_rootx()
        if 180 < new_width < 700:
            self.sidebar.configure(width=new_width)
            self.grid_columnconfigure(0, minsize=new_width)

    def select_folder(self):
        path = filedialog.askdirectory()
        if path:
            self.folder_path = os.path.normpath(path)
            self.update_chat("System", f"フォルダ設定: {self.folder_path}")

    def update_chat(self, sender, text):
        self.chat_display.configure(state="normal")
        self.chat_display.insert("end", f"【{sender}】\n{text}\n\n")
        self.chat_display.configure(state="disabled")
        self.chat_display.see("end")

    def request_stop(self):
        self.stop_requested = True
        self.update_chat("System", "中断リクエスト中...")

    def start_scan(self):
        if not self.folder_path:
            messagebox.showwarning("Warning", "フォルダを選択してください")
            return
        self.is_indexing = True
        self.stop_requested = False
        self.error_files = []
        self.error_label.configure(text="")
        self.btn_scan.configure(state="disabled")
        self.btn_stop.configure(state="normal")
        threading.Thread(target=self.indexing_task, daemon=True).start()

    def indexing_task(self):
        db = None
        try:
            embed_model, db_dir = self.get_model_config()

            self.after(0, lambda: self.file_name_label.configure(text="⏳ モデルをロード/確認中...", text_color="#AAAAAA"))
            try:
                embeddings = self.create_embeddings(embed_model)
            except Exception as e:
                err_msg = f"モデルの取得に失敗しました。プロキシ設定を確認してください。\n{str(e)}"
                self.after(0, lambda m=err_msg: self.show_error_and_reset(m))
                return

            indexed_files = set()
            if os.path.exists(db_dir):
                tmp_db = Chroma(persist_directory=db_dir, embedding_function=embeddings)
                data = tmp_db.get()
                if data and 'metadatas' in data:
                    indexed_files = {os.path.normpath(m['source']) for m in data['metadatas'] if m and 'source' in m}
                del tmp_db

            valid_exts = ('.pdf', '.pptx', '.docx', '.xlsx', '.txt')
            all_files = [os.path.join(r, f) for r, _, fs in os.walk(self.folder_path) for f in fs if f.lower().endswith(valid_exts)]
            target_files = [f for f in all_files if os.path.normpath(f) not in indexed_files]
            
            if not target_files:
                self.update_chat("System", "すべて最新です。")
                return

            docs = []
            for i, f in enumerate(target_files):
                if self.stop_requested: break
                fname = os.path.basename(f)
                self.after(0, lambda c=i + 1, t=len(target_files), p=int(((i + 1) / len(target_files)) * 100), n=fname: self.ui_loading(c, t, p, n))
                try:
                    ext = os.path.splitext(f)[1].lower()
                    if ext == ".pdf":
                        loader = PyMuPDFLoader(f)
                    elif ext == ".docx":
                        loader = Docx2txtLoader(f)
                    elif ext == ".txt":
                        loader = TextLoader(f, encoding="utf-8")
                    elif ext == ".xlsx":
                        loader = UnstructuredExcelLoader(f)
                    else:
                        continue
                    docs.extend(loader.load())
                except Exception as e:
                    self.error_files.append(fname)
                    self.after(0, lambda n=fname: self.error_label.configure(text=f"⚠️ Skip: {n}"))
                    continue

            if not docs and not self.stop_requested:
                self.update_chat("System", "処理可能なドキュメントがありませんでした。")
                return

            text_splitter = RecursiveCharacterTextSplitter(chunk_size=1000, chunk_overlap=100)
            raw_splits = text_splitter.split_documents(docs)
            for d in raw_splits:
                d.metadata = {k: self.clean_metadata_value(v) for k, v in d.metadata.items()}

            db = Chroma(persist_directory=db_dir, embedding_function=embeddings)
            batch_size = 30
            for i in range(0, len(raw_splits), batch_size):
                if self.stop_requested: break
                try:
                    batch = raw_splits[i : i + batch_size]
                    db.add_documents(batch)
                    done = min(i + batch_size, len(raw_splits))
                    self.after(0, lambda d=done, t=len(raw_splits), p=int((done / len(raw_splits)) * 100): self.ui_db(d, t, p))
                except Exception as e:
                    self.update_chat("Error", f"バッチ登録エラー: {str(e)}")
                    continue

            if not self.stop_requested:
                status_msg = "✅ 完了" if not self.error_files else f"✅ 完了 (スキップ {len(self.error_files)}件)"
                self.after(0, lambda m=status_msg: self.file_name_label.configure(text=m, text_color="#81C784"))
                if self.error_files:
                    self.update_chat("System", f"完了。スキップ: {', '.join(self.error_files)}")
                else:
                    self.update_chat("System", "すべてのファイルを登録しました。")
        except Exception as e:
            self.update_chat("Error", f"重大エラー: {str(e)}")
        finally:
            if db is not None:
                del db
            self.after(0, lambda: self.btn_scan.configure(state="normal"))
            self.after(0, lambda: self.btn_stop.configure(state="disabled"))

    def show_error_and_reset(self, msg):
        messagebox.showerror("Network/Model Error", msg)
        self.update_chat("Error", msg)
        self.btn_scan.configure(state="normal")
        self.btn_stop.configure(state="disabled")
        self.file_name_label.configure(text="中断されました", text_color="#EF5350")

    def ui_loading(self, c, t, p, n):
        self.load_label.configure(text=f"Loading: {c} / {t} ({p}%)")
        self.p_bar.set(c / t)
        self.file_name_label.configure(text=f"📄 {n}", text_color="#64B5F6")

    def ui_db(self, d, t, p):
        self.db_label.configure(text=f"DB登録: {d} / {t} ({p}%)")
        self.p_bar.set(d / t)

    def send_query(self):
        query = self.entry.get()
        if not query: return
        self.entry.delete(0, 'end')
        self.update_chat("You", query)
        for b in self.source_buttons: b.destroy()
        self.source_buttons = []

        # エージェントモードが有効な場合
        if self.agent_switch.get():
            threading.Thread(target=self.agent_rag_task, args=(query,), daemon=True).start()
        else:
            threading.Thread(target=self.rag_task, args=(query,), daemon=True).start()

    def agent_rag_task(self, query):
        """エージェントモード: AIが自律的に必要な資料を判断して検索"""
        db = None
        try:
            # ステップ1: 質問を分析して検索キーワードを生成
            self.after(0, lambda: self.update_chat("Agent", "🤔 質問を分析中..."))

            embed_model, db_dir = self.get_model_config()

            try:
                embeddings = self.create_embeddings(embed_model)
                db = Chroma(persist_directory=db_dir, embedding_function=embeddings)
            except Exception as e:
                self.after(0, lambda: self.update_chat("Error", f"Embeddingロード失敗: {str(e)}"))
                return

            # AIに質問を分析させて検索キーワードを抽出
            llm = Ollama(model=self.ollama_model)
            analysis_prompt = f"""以下の質問に答えるために、どのような資料を検索すべきか分析してください。
検索キーワードを3つ提案してください（カンマ区切り）。

質問: {query}

検索キーワード:"""

            try:
                keywords_response = llm.invoke(analysis_prompt)
                keywords = [k.strip() for k in keywords_response.split(',')[:3]]
                self.after(0, lambda k=keywords: self.update_chat("Agent", f"💡 検索キーワード: {', '.join(k)}"))
            except:
                keywords = [query]  # フォールバック

            # ステップ2: 複数のキーワードで検索して資料を収集
            all_docs = []
            seen_sources = set()

            for keyword in keywords:
                self.after(0, lambda k=keyword: self.update_chat("Agent", f"🔍 「{k}」で検索中..."))

                initial_k = 5 if (self.rerank_switch.get() and HAS_RERANKER) else 3
                docs = db.as_retriever(search_kwargs={"k": initial_k}).invoke(keyword)

                # 重複を除外して追加
                for doc in docs:
                    source = doc.metadata.get('source', '')
                    if source not in seen_sources:
                        all_docs.append(doc)
                        seen_sources.add(source)

            if not all_docs:
                self.after(0, lambda: self.update_chat("Agent", "⚠️ 関連資料が見つかりませんでした。"))
                return

            self.after(0, lambda: self.update_chat("Agent", f"📚 {len(all_docs)}件の資料を発見"))

            # ステップ3: リランキング
            if self.rerank_switch.get() and HAS_RERANKER and len(all_docs) > 3:
                self.after(0, lambda: self.update_chat("Agent", "🎯 最も関連性の高い資料を選定中..."))
                try:
                    reranker = CrossEncoder(
                        "BAAI/bge-reranker-base",
                        device='cpu',
                        cache_dir=MODELS_DIR
                    )
                    pairs = [[query, d.page_content] for d in all_docs]
                    scores = reranker.predict(pairs)
                    all_docs = [d for _, d in sorted(zip(scores, all_docs), key=lambda x: x[0], reverse=True)[:5]]
                except Exception as e:
                    self.after(0, lambda: self.update_chat("Agent", f"⚠️ リランカー失敗: {str(e)}"))
                    all_docs = all_docs[:5]

            # ステップ4: 資料の内容を確認して、十分な情報があるか判断
            context = "\n\n".join([f"【出典: {os.path.basename(d.metadata.get('source', ''))}】\n{d.page_content}" for d in all_docs])

            self.after(0, lambda: self.update_chat("Agent", "🧠 資料を読み込んで回答を生成中..."))

            # 資料が十分か判断
            check_prompt = f"""以下の資料を使って、この質問に答えられますか？
「はい」または「いいえ」だけで答えてください。

質問: {query}

資料:
{context[:2000]}...

回答:"""

            try:
                sufficiency = llm.invoke(check_prompt).strip().lower()
                if "いいえ" in sufficiency or "no" in sufficiency:
                    self.after(0, lambda: self.update_chat("Agent", "⚠️ 資料が不十分です。別の検索キーワードを試します..."))
                    # 再検索ロジックをここに追加可能
            except:
                pass

            # ステップ5: 最終的な回答を生成
            try:
                prompt = ChatPromptTemplate.from_template(
                    "以下の資料を参考に、質問に詳しく答えてください。"
                    "資料のどの部分を参照したかも明示してください。\n\n"
                    "資料:\n{context}\n\n質問: {question}"
                )
                chain = prompt | llm | StrOutputParser()
                res = chain.invoke({"context": context, "question": query})
                self.after(0, lambda r=res: self.update_chat("Assistant", r))
            except Exception as e:
                self.after(0, lambda: self.update_chat("Error", f"Ollamaエラー。起動を確認してください。\n{str(e)}"))
                return

            # 参照資料を表示
            for d in all_docs:
                p, pg = d.metadata.get("source"), d.metadata.get("page")
                self.after(0, lambda f=p, g=pg: self.add_source_button(f, g))

        except Exception as e:
            self.after(0, lambda: self.update_chat("Error", f"エージェントエラー: {str(e)}"))
        finally:
            if db is not None:
                del db

    def rag_task(self, query):
        db = None
        try:
            self.after(0, lambda: self.update_chat("Assistant", "資料を検索中..."))
            embed_model, db_dir = self.get_model_config()

            try:
                embeddings = self.create_embeddings(embed_model)
                db = Chroma(persist_directory=db_dir, embedding_function=embeddings)
            except Exception as e:
                self.after(0, lambda: self.update_chat("Error", f"Embeddingロード失敗: {str(e)}"))
                return

            initial_k = 10 if (self.rerank_switch.get() and HAS_RERANKER) else 3
            docs = db.as_retriever(search_kwargs={"k": initial_k}).invoke(query)

            if self.rerank_switch.get() and HAS_RERANKER:
                self.after(0, lambda: self.update_chat("Assistant", "リランカーを準備中 (初回はDL発生)..."))
                try:
                    reranker = CrossEncoder(
                        "BAAI/bge-reranker-base",
                        device='cpu',
                        cache_dir=MODELS_DIR
                    )
                    self.after(0, lambda: self.update_chat("Assistant", "内容を精査中..."))
                    pairs = [[query, d.page_content] for d in docs]
                    scores = reranker.predict(pairs)
                    docs = [d for _, d in sorted(zip(scores, docs), key=lambda x: x[0], reverse=True)[:3]]
                except Exception as e:
                    self.after(0, lambda: self.update_chat("Error", f"リランカーの失敗: {str(e)}"))
                    docs = docs[:3]

            context = "\n\n".join([f"【出典: {os.path.basename(d.metadata.get('source', ''))}】\n{d.page_content}" for d in docs])
            
            try:
                llm = Ollama(model=self.ollama_model)
                prompt = ChatPromptTemplate.from_template("資料を参考に日本語で答えてください。\n資料:\n{context}\n\n質問: {question}")
                chain = prompt | llm | StrOutputParser()
                res = chain.invoke({"context": context, "question": query})
                self.after(0, lambda r=res: self.update_chat("Assistant", r))
            except Exception as e:
                self.after(0, lambda: self.update_chat("Error", f"Ollamaエラー。起動を確認してください。\n{str(e)}"))
                return

            for d in docs:
                p, pg = d.metadata.get("source"), d.metadata.get("page")
                self.after(0, lambda f=p, g=pg: self.add_source_button(f, g))
        except Exception as e:
            self.after(0, lambda: self.update_chat("Error", str(e)))
        finally:
            if db is not None:
                del db

    def open_file_safely(self, file_path):
        """ファイルを安全に開く"""
        if os.path.exists(file_path):
            try:
                os.startfile(file_path)
            except Exception as e:
                messagebox.showerror("Error", f"ファイルを開けません: {str(e)}")
        else:
            messagebox.showerror("Error", "ファイルが見つかりません")

    def add_source_button(self, file_path, page_num=None):
        fname = os.path.basename(file_path)
        lbl = f"📄 {fname}" + (f" (P.{page_num + 1})" if page_num is not None else "")
        btn = ctk.CTkButton(
            self.source_frame,
            text=lbl,
            anchor="w",
            font=self.font_mini,
            fg_color="#333333",
            height=32,
            command=lambda p=file_path: self.open_file_safely(p)
        )
        btn.pack(fill="x", pady=2)
        self.source_buttons.append(btn)

if __name__ == "__main__":
    app = RAGWinApp()
    app.mainloop()