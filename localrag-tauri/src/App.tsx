import { useState, useEffect, useCallback, useMemo } from 'react';
import { Sidebar } from './components/Sidebar';
import { ChatArea } from './components/ChatArea';
import { useBackend } from './hooks/useBackend';
import { useUpdater } from './hooks/useUpdater';
import type { Message, SourceInfo, IndexStats, IndexProgress, ModelsPayload } from './types';

function App() {
  // Ollama state
  const [ollamaRunning, setOllamaRunning] = useState(false);

  // Model state
  const [llmModels, setLlmModels] = useState<string[]>(['gemma3:4b']);
  const [embeddingModels, setEmbeddingModels] = useState<string[]>(['nomic-embed-text']);
  const [currentLlmModel, setCurrentLlmModel] = useState('gemma3:4b');
  const [currentEmbeddingModel, setCurrentEmbeddingModel] = useState('nomic-embed-text');

  // Index state
  const [folderPath, setFolderPath] = useState<string | null>(null);
  const [isIndexing, setIsIndexing] = useState(false);
  const [indexProgress, setIndexProgress] = useState(0);
  const [currentFile, setCurrentFile] = useState('');
  const [indexStats, setIndexStats] = useState<IndexStats | null>(null);

  // Chat state
  const [messages, setMessages] = useState<Message[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);
  const [agentMode, setAgentMode] = useState(false);
  const [agentProgress, setAgentProgress] = useState('');
  const [currentSources, setCurrentSources] = useState<SourceInfo[]>([]);

  // Error state
  const [error, setError] = useState<string | null>(null);

  // Updater
  const { checkForUpdates, downloadAndInstall } = useUpdater();

  // Backend callbacks
  const callbacks = useMemo(() => ({
    onOllamaStatus: (status: boolean) => setOllamaRunning(status),
    onFolderSelected: (path: string) => setFolderPath(path),
    onIndexProgress: (progress: IndexProgress) => {
      setIndexProgress(progress.progress);
      setCurrentFile(progress.file);
    },
    onIndexStatsUpdate: (stats: IndexStats) => setIndexStats(stats),
    onIndexComplete: (stats: IndexStats) => {
      setIndexStats(stats);
      setIsIndexing(false);
      setIndexProgress(0);
      setCurrentFile('');
    },
    onIndexingCancelled: () => {
      setIsIndexing(false);
      setIndexProgress(0);
      setCurrentFile('');
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
      setEmbeddingModels(models.embedding_models.length > 0 ? models.embedding_models : ['nomic-embed-text']);
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

  const handleCheckUpdates = useCallback(async () => {
    const update = await checkForUpdates();
    if (update) {
      if (confirm(`Update ${update.version} available. Download and install?`)) {
        await downloadAndInstall();
      }
    } else {
      alert('No updates available.');
    }
  }, [checkForUpdates, downloadAndInstall]);

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
        folderPath={folderPath}
        isIndexing={isIndexing}
        indexProgress={indexProgress}
        currentFile={currentFile}
        indexStats={indexStats}
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
        onCheckUpdates={handleCheckUpdates}
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
    </div>
  );
}

export default App;
