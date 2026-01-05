#!/usr/bin/env python3
"""
ChromaDB HTTP Bridge Server

ChromaDB 1.4.0用のHTTPサーバーブリッジ。
Rust版GUIから接続できるようにRESTful APIを提供します。
"""

import os
import sys
from pathlib import Path
from typing import Dict, List, Any

try:
    import chromadb
    from chromadb.config import Settings
except ImportError:
    print("ERROR: chromadb not installed")
    print("Install with: pip install chromadb")
    sys.exit(1)

try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import JSONResponse
    import uvicorn
    from pydantic import BaseModel
except ImportError:
    print("ERROR: FastAPI not installed")
    print("Install with: pip install fastapi uvicorn")
    sys.exit(1)

# 設定
CHROMA_DB_PATH = Path(__file__).parent / "chroma_db"
PORT = 8001  # ポート8000は既に使用中のため8001を使用

print("=" * 60)
print("  ChromaDB HTTP Bridge Server")
print("=" * 60)
print(f"ChromaDB version: {chromadb.__version__}")
print(f"Database path: {CHROMA_DB_PATH}")
print(f"Server URL: http://localhost:{PORT}")
print("=" * 60)

# ChromaDBクライアント初期化
client = chromadb.PersistentClient(
    path=str(CHROMA_DB_PATH),
    settings=Settings(anonymized_telemetry=False)
)

# FastAPI app
app = FastAPI(title="ChromaDB Bridge", version="1.0.0")


# Pydanticモデル
class AddDocumentsRequest(BaseModel):
    embeddings: List[List[float]]
    documents: List[str]
    metadatas: List[Dict[str, str]]
    ids: List[str] = None


class QueryRequest(BaseModel):
    query_embeddings: List[List[float]]
    n_results: int = 10
    where_filter: Dict[str, Any] = None


class GetDocumentsRequest(BaseModel):
    limit: int = 1000
    ids: List[str] = None
    where: Dict[str, Any] = None


@app.get("/")
def root():
    return {"status": "ok", "message": "ChromaDB Bridge Server", "version": chromadb.__version__}


@app.get("/api/v1/heartbeat")
def heartbeat():
    """ヘルスチェック"""
    return {"status": "ok"}


@app.get("/api/v1/collections/{collection_name}")
def get_collection(collection_name: str):
    """コレクション情報取得"""
    try:
        collection = client.get_or_create_collection(
            name=collection_name,
            metadata={"hnsw:space": "cosine"}
        )
        return {
            "name": collection_name,
            "id": collection_name,  # IDは名前と同じにする
            "metadata": collection.metadata
        }
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.post("/api/v1/collections/{collection_name}/add")
def add_documents(collection_name: str, request: AddDocumentsRequest):
    """ドキュメント追加"""
    try:
        collection = client.get_or_create_collection(
            name=collection_name,
            metadata={"hnsw:space": "cosine"}
        )

        # IDが指定されていない場合は自動生成
        ids = request.ids
        if not ids:
            import uuid
            ids = [str(uuid.uuid4()) for _ in range(len(request.documents))]

        collection.add(
            embeddings=request.embeddings,
            documents=request.documents,
            metadatas=request.metadatas,
            ids=ids
        )

        return {"status": "ok", "added": len(request.documents)}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/collections/{collection_name}/query")
def query_documents(collection_name: str, request: QueryRequest):
    """ドキュメント検索"""
    try:
        collection = client.get_or_create_collection(
            name=collection_name,
            metadata={"hnsw:space": "cosine"}
        )

        results = collection.query(
            query_embeddings=request.query_embeddings,
            n_results=request.n_results,
            where=request.where_filter
        )

        return results
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/collections/{collection_name}/get")
def get_documents(collection_name: str, request: GetDocumentsRequest):
    """全ドキュメント取得（POSTメソッド）"""
    try:
        collection = client.get_or_create_collection(
            name=collection_name,
            metadata={"hnsw:space": "cosine"}
        )

        # パラメータを設定
        kwargs = {"limit": request.limit}
        if request.ids:
            kwargs["ids"] = request.ids
        if request.where:
            kwargs["where"] = request.where

        results = collection.get(**kwargs)

        return results
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/api/v1/collections/{collection_name}/count")
def count_documents(collection_name: str):
    """ドキュメント数取得"""
    try:
        collection = client.get_or_create_collection(
            name=collection_name,
            metadata={"hnsw:space": "cosine"}
        )

        return {"count": collection.count()}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    print()
    print("Starting server...")
    print("Press Ctrl+C to stop")
    print()

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=PORT,
        log_level="info"
    )
