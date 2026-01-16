import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { Sidebar } from './components/Sidebar';
import { ChatArea } from './components/ChatArea';
import { OllamaSetupGuide } from './components/OllamaSetupGuide';
import { IndexAnalysisModal } from './components/IndexAnalysisModal';
import { useBackend } from './hooks/useBackend';
import type { Message, SourceInfo, IndexStats, IndexProgress, ModelsPayload, OllamaStatusInfo, SystemInfo, IndexAnalysis } from './types';

function App() {
  // Ollama state - null means "checking"
  const [ollamaRunning, setOllamaRunning] = useState<boolean | null>(null);
  const [ollamaInstalled, setOllamaInstalled] = useState<boolean | null>(null); // null = checking
  const [showSetupGuide, setShowSetupGuide] = useState(false);

  // System info state
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  // Model state
  const [llmModels, setLlmModels] = useState<string[]>(['gemma3:4b']);
  const [embeddingModels, setEmbeddingModels] = useState<string[]>(['bge-m3']);
  const [currentLlmModel, setCurrentLlmModel] = useState('gemma3:4b');
  const [currentEmbeddingModel, setCurrentEmbeddingModel] = useState('bge-m3');

  // Index state
  const [folderPath, setFolderPath] = useState<string | null>(null);
  const [isIndexing, setIsIndexing] = useState(false);
  const [indexProgress, setIndexProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [indexPhase, setIndexPhase] = useState<'loading' | 'splitting' | 'embedding' | 'storing' | null>(null);
  const [indexCurrent, setIndexCurrent] = useState(0);
  const [indexTotal, setIndexTotal] = useState(0);
  const [indexStats, setIndexStats] = useState<IndexStats | null>(null);

  // Chat state
  const [messages, setMessages] = useState<Message[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [agentMode, setAgentMode] = useState(false);
  const [agentProgress, setAgentProgress] = useState('');
  const [currentSources, setCurrentSources] = useState<SourceInfo[]>([]);

  // Error state
  const [error, setError] = useState<string | null>(null);

  // Analysis modal state
  const [showAnalysis, setShowAnalysis] = useState(false);
  const [indexAnalysis, setIndexAnalysis] = useState<IndexAnalysis | null>(null);

  // Ref to track indexing state for callbacks
  const isIndexingRef = useRef(false);

  // Keep ref in sync with state
  useEffect(() => {
    isIndexingRef.current = isIndexing;
  }, [isIndexing]);

  // Backend callbacks
  const callbacks = useMemo(() => ({
    onOllamaStatus: (status: boolean) => {
      // Skip Ollama status updates during indexing to prevent UI flickering
      if (isIndexingRef.current) return;
      setOllamaRunning(status);
    },
    onOllamaStatusInfo: (status: OllamaStatusInfo) => {
      // Skip Ollama status updates during indexing to prevent UI flickering
      if (isIndexingRef.current) return;
      setOllamaInstalled(status.installed);
      setOllamaRunning(status.running);
      // Show setup guide if not installed (first time only)
      if (!status.installed) {
        setShowSetupGuide(true);
      }
    },
    onFolderSelected: (path: string) => setFolderPath(path),
    onIndexProgress: (progress: IndexProgress) => {
      setIndexProgress(progress.progress);
      setCurrentFile(progress.file);
      if (progress.phase) {
        setIndexPhase(progress.phase);
      }
      if (progress.current !== undefined) {
        setIndexCurrent(progress.current);
      }
      if (progress.total !== undefined) {
        setIndexTotal(progress.total);
      }
    },
    onIndexStatsUpdate: (stats: IndexStats) => setIndexStats(stats),
    onIndexComplete: (stats: IndexStats) => {
      console.log('Index complete received:', stats);
      setIndexStats(stats);
      setIsIndexing(false);
      setIndexProgress(0);
      setCurrentFile('');
      setIndexPhase(null);
      setIndexCurrent(0);
      setIndexTotal(0);
    },
    onQueryChunk: (chunk: string) => {
      setMessages((prev) => {
        const lastMsg = prev[prev.length - 1];
        if (lastMsg && lastMsg.role === 'assistant') {
          return [
            ...prev.slice(0, -1),
            { ...lastMsg, content: lastMsg.content + chunk },
          ];
        }
        return prev;
      });
    },
    onQueryComplete: (sources: SourceInfo[]) => {
      setCurrentSources(sources);
      setIsGenerating(false);
      setAgentProgress('');
    },
    onAgentProgress: (message: string) => setAgentProgress(message),
    onModelsRefreshed: (models: ModelsPayload) => {
      setLlmModels(models.llm_models.length > 0 ? models.llm_models : ['gemma3:4b']);
      setEmbeddingModels(models.embedding_models.length > 0 ? models.embedding_models : ['bge-m3']);
    },
    onError: (err: string) => {
      setError(err);
      setIsGenerating(false);
      setAgentProgress('');
      setTimeout(() => setError(null), 5000);
    },
  }), []);

  const backend = useBackend(callbacks);

  // Initialize
  useEffect(() => {
    backend.getCurrentModels().then((models) => {
      setCurrentLlmModel(models.llm_model);
      setCurrentEmbeddingModel(models.embedding_model);
    });
    backend.refreshModels();

    // Get system info on startup (CPU info only, Ollama status comes from background checker)
    backend.getSystemInfo().then((info) => {
      setSystemInfo(info);
    }).catch(() => {
      // Ignore errors on startup
    });
  }, [backend]);

  // Handlers
  const handleSelectFolder = useCallback(async () => {
    const path = await backend.selectFolder();
    if (path) {
      setFolderPath(path);
    }
  }, [backend]);

  const handleStartIndexing = useCallback(async () => {
    if (folderPath) {
      setIsIndexing(true);
      await backend.startIndexing(folderPath);
    }
  }, [backend, folderPath]);

  const handleStopIndexing = useCallback(async () => {
    await backend.stopIndexing();
  }, [backend]);

  const handleSendMessage = useCallback(async (text: string) => {
    // Add user message
    setMessages((prev) => [...prev, { role: 'user', content: text }]);

    // Add empty assistant message for streaming
    setMessages((prev) => [...prev, { role: 'assistant', content: '' }]);

    setIsGenerating(true);
    setCurrentSources([]);
    setAgentProgress('');

    await backend.sendQuery(text, agentMode);
  }, [backend, agentMode]);

  const handleToggleAgentMode = useCallback(() => {
    setAgentMode((prev) => !prev);
  }, []);

  const handleSetLlmModel = useCallback(async (model: string) => {
    setCurrentLlmModel(model);
    await backend.setLlmModel(model);
  }, [backend]);

  const handleSetEmbeddingModel = useCallback(async (model: string) => {
    setCurrentEmbeddingModel(model);
    await backend.setEmbeddingModel(model);
  }, [backend]);

  const handleRefreshStats = useCallback(async () => {
    const stats = await backend.getIndexStats();
    if (stats) {
      setIndexStats(stats);
    }
  }, [backend]);

  const handleAnalyze = useCallback(async () => {
    try {
      const analysis = await backend.analyzeIndex();
      setIndexAnalysis(analysis);
      setShowAnalysis(true);
    } catch (err) {
      setError('Failed to analyze index');
      setTimeout(() => setError(null), 5000);
    }
  }, [backend]);

  return (
    <div className="flex h-screen">
      {/* Error Toast */}
      {error && (
        <div className="fixed top-4 right-4 bg-error text-text-bright px-4 py-2 rounded-lg shadow-lg z-50 animate-pulse">
          {error}
        </div>
      )}

      <Sidebar
        ollamaRunning={ollamaRunning}
        ollamaInstalled={ollamaInstalled}
        folderPath={folderPath}
        isIndexing={isIndexing}
        indexProgress={indexProgress}
        currentFile={currentFile}
        indexPhase={indexPhase}
        indexCurrent={indexCurrent}
        indexTotal={indexTotal}
        indexStats={indexStats}
        systemInfo={systemInfo}
        llmModels={llmModels}
        embeddingModels={embeddingModels}
        currentLlmModel={currentLlmModel}
        currentEmbeddingModel={currentEmbeddingModel}
        onSelectFolder={handleSelectFolder}
        onStartIndexing={handleStartIndexing}
        onStopIndexing={handleStopIndexing}
        onRefreshModels={backend.refreshModels}
        onSetLlmModel={handleSetLlmModel}
        onSetEmbeddingModel={handleSetEmbeddingModel}
        onShowSetupGuide={() => setShowSetupGuide(true)}
        onRefreshStats={handleRefreshStats}
        onAnalyze={handleAnalyze}
      />

      <ChatArea
        messages={messages}
        isGenerating={isGenerating}
        agentMode={agentMode}
        agentProgress={agentProgress}
        currentSources={currentSources}
        onSendMessage={handleSendMessage}
        onToggleAgentMode={handleToggleAgentMode}
      />

      {/* Ollama Setup Guide Modal */}
      {showSetupGuide && (
        <OllamaSetupGuide
          onClose={() => setShowSetupGuide(false)}
          systemInfo={systemInfo}
          ollamaRunning={ollamaRunning}
        />
      )}

      {/* Index Analysis Modal */}
      {showAnalysis && indexAnalysis && (
        <IndexAnalysisModal
          analysis={indexAnalysis}
          onClose={() => setShowAnalysis(false)}
        />
      )}
    </div>
  );
}

export default App;
