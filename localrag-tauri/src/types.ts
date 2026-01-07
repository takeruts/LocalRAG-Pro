export interface Message {
  role: 'user' | 'assistant';
  content: string;
}

export interface SourceInfo {
  source: string;
  page: number | null;
  score: number;
}

export interface IndexStats {
  total_files: number;
  indexed_files: number;
  skipped_files: number;
  total_chunks: number;
  total_embeddings: number;
  indexed_folder: string | null;
}

export interface IndexProgress {
  progress: number;
  file: string;
}

export interface ModelsPayload {
  llm_models: string[];
  embedding_models: string[];
}

export interface CurrentModels {
  llm_model: string;
  embedding_model: string;
}
