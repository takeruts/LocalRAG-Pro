import type { IndexStats, SystemInfo } from '../types';

type IndexPhase = 'loading' | 'splitting' | 'embedding' | 'storing';

interface SidebarProps {
  ollamaRunning: boolean | null; // null = checking
  ollamaInstalled: boolean | null; // null = checking
  folderPath: string | null;
  isIndexing: boolean;
  indexProgress: number;
  currentFile: string;
  indexPhase: IndexPhase | null;
  indexCurrent: number;
  indexTotal: number;
  indexStats: IndexStats | null;
  systemInfo: SystemInfo | null;
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
  onShowSetupGuide: () => void;
  onRefreshStats: () => void;
  onAnalyze: () => void;
}

function getPhaseLabel(phase: IndexPhase): { label: string; color: string } {
  switch (phase) {
    case 'loading':
      return { label: 'Loading Files', color: 'text-blue-400' };
    case 'splitting':
      return { label: 'Splitting', color: 'text-purple-400' };
    case 'embedding':
      return { label: 'Embedding', color: 'text-yellow-400' };
    case 'storing':
      return { label: 'Storing', color: 'text-green-400' };
  }
}

export function Sidebar({
  ollamaRunning,
  ollamaInstalled,
  folderPath,
  isIndexing,
  indexProgress,
  currentFile,
  indexPhase,
  indexCurrent,
  indexTotal,
  indexStats,
  systemInfo,
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
  onShowSetupGuide,
  onRefreshStats,
  onAnalyze,
}: SidebarProps) {
  const shortenPath = (path: string, maxLen: number = 40) => {
    if (path.length <= maxLen) return path;
    const parts = path.split(/[/\\]/);
    if (parts.length <= 2) return path;
    return `${parts[0]}/.../${parts[parts.length - 1]}`;
  };

  return (
    <div className="w-[360px] min-w-[300px] max-w-[500px] bg-bg-card p-4 flex flex-col gap-4 overflow-y-auto min-h-0">
      {/* Title */}
      <div className="text-center py-2">
        <h1 className="text-xl font-bold text-primary">CPURAG</h1>
        <p className="text-xs text-text-muted">v1.0.0</p>
      </div>

      {/* CPU Info */}
      {systemInfo && (
        <div className="p-3 bg-bg-main rounded-lg">
          <h3 className="text-sm font-medium text-text-primary mb-2">CPU Info</h3>
          <div className="flex flex-col gap-1 text-xs">
            {systemInfo.cpu_name && (
              <p className="text-text-secondary truncate" title={systemInfo.cpu_name}>
                {systemInfo.cpu_name}
              </p>
            )}
            <div className="flex gap-4 text-text-muted">
              {systemInfo.cpu_cores && (
                <span>{systemInfo.cpu_cores} Cores</span>
              )}
              {systemInfo.cpu_frequency_mhz && (
                <span>{(systemInfo.cpu_frequency_mhz / 1000).toFixed(2)} GHz</span>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Ollama Status - Fixed layout to prevent flickering */}
      <div className="p-3 bg-bg-main rounded-lg">
        <div className="flex items-center gap-2 h-5">
          <div
            className={`w-3 h-3 rounded-full flex-shrink-0 ${
              ollamaInstalled === null || ollamaRunning === null
                ? 'bg-text-muted animate-pulse'
                : ollamaInstalled === false
                  ? 'bg-warning'
                  : ollamaRunning
                    ? 'bg-success'
                    : 'bg-error'
            }`}
          />
          <span className="text-sm text-text-secondary">
            Ollama: {
              ollamaInstalled === null || ollamaRunning === null
                ? 'Checking...'
                : ollamaInstalled === false
                  ? 'Not Installed'
                  : ollamaRunning
                    ? 'Running'
                    : 'Stopped'
            }
          </span>
        </div>
        {/* Fixed height message area */}
        <div className="h-6 mt-2 flex items-center">
          <p className={`text-xs ${ollamaInstalled === true && ollamaRunning === false ? 'text-text-muted' : 'text-transparent'}`}>
            Start: <code className="bg-bg-input px-1 rounded">ollama serve</code>
          </p>
        </div>
        <button
          onClick={onShowSetupGuide}
          className="w-full px-2 py-1 text-text-muted rounded text-xs hover:bg-bg-input transition-colors text-left"
        >
          Setup Guide
        </button>
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
        <div className="bg-bg-main p-3 rounded-lg flex flex-col gap-2">
          {/* Phase indicator */}
          {indexPhase && (
            <div className="flex items-center justify-between">
              <span className={`text-xs font-medium ${getPhaseLabel(indexPhase).color}`}>
                {getPhaseLabel(indexPhase).label}
              </span>
              {indexTotal > 0 && (
                <span className="text-xs text-text-muted">
                  {indexCurrent} / {indexTotal}
                </span>
              )}
            </div>
          )}

          {/* Progress bar */}
          <div className="w-full h-2 bg-bg-input rounded-full overflow-hidden">
            <div
              className={`h-full transition-all duration-300 ${
                indexPhase === 'embedding' ? 'bg-yellow-500' :
                indexPhase === 'storing' ? 'bg-green-500' :
                indexPhase === 'splitting' ? 'bg-purple-500' :
                'bg-primary'
              }`}
              style={{ width: `${indexProgress * 100}%` }}
            />
          </div>

          {/* Percentage and file info */}
          <div className="flex items-center justify-between">
            <p className="text-xs text-text-muted truncate flex-1 mr-2">{currentFile}</p>
            <span className="text-xs text-text-secondary font-mono">
              {Math.round(indexProgress * 100)}%
            </span>
          </div>

          {/* Phase steps indicator */}
          <div className="flex items-center gap-1 mt-1">
            {(['loading', 'splitting', 'embedding', 'storing'] as IndexPhase[]).map((phase, idx) => (
              <div key={phase} className="flex items-center">
                <div
                  className={`w-2 h-2 rounded-full ${
                    indexPhase === phase
                      ? getPhaseLabel(phase).color.replace('text-', 'bg-')
                      : phase === 'loading' && (indexPhase === 'splitting' || indexPhase === 'embedding' || indexPhase === 'storing')
                        ? 'bg-success'
                        : phase === 'splitting' && (indexPhase === 'embedding' || indexPhase === 'storing')
                          ? 'bg-success'
                          : phase === 'embedding' && indexPhase === 'storing'
                            ? 'bg-success'
                            : 'bg-bg-input'
                  }`}
                />
                {idx < 3 && <div className="w-4 h-0.5 bg-bg-input" />}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Index Statistics */}
      {indexStats && (
        <div className="bg-bg-main p-3 rounded-lg flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-text-primary">Index Statistics</h3>
            <div className="flex gap-2">
              <button
                onClick={onAnalyze}
                className="text-xs text-primary hover:text-primary-dim transition-colors"
                title="Analyze indexed data"
              >
                Analyze
              </button>
              <button
                onClick={onRefreshStats}
                className="text-xs text-text-muted hover:text-text-secondary transition-colors"
                title="Refresh statistics"
              >
                Refresh
              </button>
            </div>
          </div>
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
    </div>
  );
}
