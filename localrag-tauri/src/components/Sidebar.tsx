import type { IndexStats } from '../types';

interface SidebarProps {
  ollamaRunning: boolean;
  folderPath: string | null;
  isIndexing: boolean;
  indexProgress: number;
  currentFile: string;
  indexStats: IndexStats | null;
  llmModels: string[];
  embeddingModels: string[];
  currentLlmModel: string;
  currentEmbeddingModel: string;
  onSelectFolder: () => void;
  onStartIndexing: () => void;
  onStopIndexing: () => void;
  onRefreshModels: () => void;
  onSetLlmModel: (model: string) => void;
  onSetEmbeddingModel: (model: string) => void;
  onCheckUpdates: () => void;
}

export function Sidebar({
  ollamaRunning,
  folderPath,
  isIndexing,
  indexProgress,
  currentFile,
  indexStats,
  llmModels,
  embeddingModels,
  currentLlmModel,
  currentEmbeddingModel,
  onSelectFolder,
  onStartIndexing,
  onStopIndexing,
  onRefreshModels,
  onSetLlmModel,
  onSetEmbeddingModel,
  onCheckUpdates,
}: SidebarProps) {
  const shortenPath = (path: string, maxLen: number = 40) => {
    if (path.length <= maxLen) return path;
    const parts = path.split(/[/\\]/);
    if (parts.length <= 2) return path;
    return `${parts[0]}/.../${parts[parts.length - 1]}`;
  };

  return (
    <div className="w-[360px] min-w-[300px] max-w-[500px] bg-bg-card p-4 flex flex-col gap-4 overflow-y-auto">
      {/* Title */}
      <div className="text-center py-2">
        <h1 className="text-xl font-bold text-primary">LocalRAG Pro</h1>
        <p className="text-xs text-text-muted">v3.0.0</p>
      </div>

      {/* Ollama Status */}
      <div className="flex items-center gap-2 p-2 bg-bg-main rounded-lg">
        <div
          className={`w-3 h-3 rounded-full ${
            ollamaRunning ? 'bg-success' : 'bg-error'
          }`}
        />
        <span className="text-sm text-text-secondary">
          Ollama: {ollamaRunning ? 'Running' : 'Stopped'}
        </span>
      </div>

      {/* Model Selection */}
      <div className="flex flex-col gap-3">
        <div>
          <label className="text-xs text-text-muted mb-1 block">LLM Model</label>
          <div className="flex gap-2">
            <select
              value={currentLlmModel}
              onChange={(e) => onSetLlmModel(e.target.value)}
              className="flex-1 bg-bg-input text-text-primary p-2 rounded-lg text-sm border-none outline-none"
              disabled={!ollamaRunning}
            >
              {llmModels.map((model) => (
                <option key={model} value={model}>
                  {model}
                </option>
              ))}
            </select>
            <button
              onClick={onRefreshModels}
              className="px-3 py-2 bg-primary-dim text-text-bright rounded-lg text-sm hover:bg-primary transition-colors"
              disabled={!ollamaRunning}
            >
              Refresh
            </button>
          </div>
        </div>

        <div>
          <label className="text-xs text-text-muted mb-1 block">Embedding Model</label>
          <select
            value={currentEmbeddingModel}
            onChange={(e) => onSetEmbeddingModel(e.target.value)}
            className="w-full bg-bg-input text-text-primary p-2 rounded-lg text-sm border-none outline-none"
            disabled={!ollamaRunning}
          >
            {embeddingModels.map((model) => (
              <option key={model} value={model}>
                {model}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Folder Selection */}
      <div className="flex flex-col gap-2">
        <label className="text-xs text-text-muted">Document Folder</label>
        <button
          onClick={onSelectFolder}
          className="w-full p-3 bg-bg-input text-text-secondary rounded-lg text-sm text-left hover:bg-bg-main transition-colors"
        >
          {folderPath ? shortenPath(folderPath) : 'Click to select folder...'}
        </button>
      </div>

      {/* Indexing Controls */}
      <div className="flex gap-2">
        <button
          onClick={onStartIndexing}
          disabled={!folderPath || isIndexing || !ollamaRunning}
          className={`flex-1 p-3 rounded-lg text-sm font-medium transition-colors ${
            !folderPath || isIndexing || !ollamaRunning
              ? 'bg-bg-input text-text-muted cursor-not-allowed'
              : 'bg-success text-text-bright hover:opacity-90'
          }`}
        >
          {isIndexing ? 'Indexing...' : 'Start Indexing'}
        </button>
        {isIndexing && (
          <button
            onClick={onStopIndexing}
            className="px-4 py-3 bg-error text-text-bright rounded-lg text-sm font-medium hover:opacity-90 transition-colors"
          >
            Stop
          </button>
        )}
      </div>

      {/* Progress Bar */}
      {isIndexing && (
        <div className="flex flex-col gap-2">
          <div className="w-full h-2 bg-bg-input rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all duration-300"
              style={{ width: `${indexProgress * 100}%` }}
            />
          </div>
          <p className="text-xs text-text-muted truncate">{currentFile}</p>
        </div>
      )}

      {/* Index Statistics */}
      {indexStats && (
        <div className="bg-bg-main p-3 rounded-lg flex flex-col gap-2">
          <h3 className="text-sm font-medium text-text-primary">Index Statistics</h3>
          {indexStats.indexed_folder && (
            <p className="text-xs text-primary truncate" title={indexStats.indexed_folder}>
              {shortenPath(indexStats.indexed_folder)}
            </p>
          )}
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div>
              <span className="text-text-muted">Files:</span>
              <span className="text-text-primary ml-1">{indexStats.indexed_files}</span>
            </div>
            <div>
              <span className="text-text-muted">Chunks:</span>
              <span className="text-text-primary ml-1">{indexStats.total_chunks}</span>
            </div>
            <div>
              <span className="text-text-muted">Embeddings:</span>
              <span className="text-text-primary ml-1">{indexStats.total_embeddings}</span>
            </div>
            <div>
              <span className="text-text-muted">Skipped:</span>
              <span className="text-text-primary ml-1">{indexStats.skipped_files}</span>
            </div>
          </div>
        </div>
      )}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Update Check */}
      <button
        onClick={onCheckUpdates}
        className="w-full p-2 bg-bg-input text-text-secondary rounded-lg text-xs hover:bg-bg-main transition-colors"
      >
        Check for Updates
      </button>
    </div>
  );
}
