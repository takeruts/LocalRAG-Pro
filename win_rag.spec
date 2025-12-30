# -*- mode: python ; coding: utf-8 -*-
from PyInstaller.utils.hooks import collect_all
import os

# データファイルの収集
datas = []
binaries = []
hiddenimports = []

# CustomTkinterの収集
try:
    import customtkinter
    ctk_path = os.path.dirname(customtkinter.__file__)
    datas.append((ctk_path, 'customtkinter'))
except ImportError:
    print("Warning: customtkinter not found")

# 必要なパッケージを収集
packages_to_collect = [
    'langchain_community',
    'langchain_huggingface',
    'chromadb',
    'sentence_transformers',
    'transformers',
    'tokenizers',
    'huggingface_hub',
    'sklearn',
    'scipy',
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
    # tiktoken
    'tiktoken_ext.openai_public',
    'tiktoken_ext',

    # PIL/Pillow
    'PIL._tkinter_finder',
    'PIL.Image',

    # sentence_transformers - 全サブモジュール
    'sentence_transformers',
    'sentence_transformers.cross_encoder',
    'sentence_transformers.cross_encoder.CrossEncoder',
    'sentence_transformers.models',
    'sentence_transformers.models.Transformer',
    'sentence_transformers.models.Pooling',
    'sentence_transformers.models.Dense',
    'sentence_transformers.models.Normalize',
    'sentence_transformers.evaluation',
    'sentence_transformers.util',
    'sentence_transformers.SentenceTransformer',

    # transformers
    'transformers',
    'transformers.models',
    'transformers.models.auto',
    'transformers.models.bert',
    'transformers.tokenization_utils',
    'transformers.tokenization_utils_base',

    # tokenizers
    'tokenizers',
    'tokenizers.implementations',

    # torch
    'torch',
    'torch.nn',
    'torch.nn.functional',

    # numpy
    'numpy',
    'numpy.core',
    'numpy.core._multiarray_umath',

    # sklearn (sentence_transformersが使用)
    'sklearn',
    'sklearn.metrics',
    'sklearn.metrics.pairwise',
    'sklearn.utils',
    'sklearn.utils._param_validation',

    # scipy (sklearnが依存)
    'scipy',
    'scipy.sparse',
    'scipy.sparse.csgraph',
    'scipy.special',
    'scipy.linalg',
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
        'matplotlib',
        'pytest',
        'notebook',
        'jupyter',
        'IPython',
        'pandas.plotting',
        'pandas.tests',
    ],
    noarchive=False,
    optimize=0,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='LocalRAG-Pro',
    debug=True,  # デバッグモード有効
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,  # コンソールを表示してエラー確認
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='LocalRAG-Pro',
)
