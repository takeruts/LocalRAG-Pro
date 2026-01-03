# -*- mode: python ; coding: utf-8 -*-
from PyInstaller.utils.hooks import collect_all
import os

# データファイルの収集
datas = []
binaries = []
hiddenimports = []

# 必要なパッケージを収集（Ollamaのみ版 - 軽量化）
packages_to_collect = [
    'customtkinter',
    'langchain_community',
    'langchain_core',
    'langchain_text_splitters',
    'chromadb',
]

for package in packages_to_collect:
    try:
        tmp_ret = collect_all(package)
        datas += tmp_ret[0]
        binaries += tmp_ret[1]
        hiddenimports += tmp_ret[2]
        print(f"Successfully collected {package}")
    except Exception as e:
        print(f"Warning: Could not collect {package}: {e}")

# 追加の隠しインポート
hiddenimports += [
    # CustomTkinter
    'customtkinter',

    # LangChain
    'langchain_community.llms',
    'langchain_community.llms.ollama',
    'langchain_community.document_loaders',
    'langchain_community.vectorstores',
    'langchain_community.vectorstores.chroma',
    'langchain_core.prompts',
    'langchain_core.output_parsers',
    'langchain_core.documents',
    'langchain_text_splitters',
    'langchain_community.embeddings',
    'langchain_community.embeddings.ollama',

    # numpy
    'numpy',
    'numpy.core._multiarray_umath',
]

a = Analysis(
    ['win_rag.py'],
    pathex=[],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # 開発・テストツール
        'matplotlib',
        'pytest',
        'notebook',
        'jupyter',
        'IPython',
        'debugpy',
        'pdb',
        'unittest',
        'test',
        'tests',
        '_pytest',

        # 不要なデータサイエンスライブラリ
        'pandas',
        'plotly',
        'seaborn',
        'bokeh',

        # 画像処理
        'cv2',
        'PIL.ImageQt',
        'PIL.ImageShow',

        # Torch/Transformers関連（完全除外）
        'torch',
        'transformers',
        'tokenizers',
        'accelerate',
        'bitsandbytes',
        'sentence_transformers',

        # 開発用ツール
        'black',
        'mypy',
        'pylint',
        'flake8',
    ],
    noarchive=False,
    optimize=2,  # Python最適化レベル（0→2）
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='LocalRAG-Pro',
    debug=False,  # リリースモード
    bootloader_ignore_signals=False,
    strip=False,  # Windowsではstripツールが無いためFalse
    upx=True,    # UPX圧縮有効
    console=False,  # コンソールなし
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,  # アイコン追加可能
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,  # Windowsではstripツールが無いためFalse
    upx=True,    # 全バイナリをUPX圧縮
    upx_exclude=[
        # UPX圧縮で問題が起きる可能性のあるファイルを除外
        'vcruntime*.dll',
        'python*.dll',
        'Qt*.dll',
    ],
    name='LocalRAG-Pro',
)
