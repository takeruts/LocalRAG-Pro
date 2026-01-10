import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useCallback } from 'react';
import type { IndexStats, IndexProgress, SourceInfo, ModelsPayload, CurrentModels, OllamaStatusInfo, SystemInfo, FullSystemStatus } from '../types';

export function useBackend(callbacks: {
  onOllamaStatus?: (status: boolean) => void;
  onOllamaStatusInfo?: (status: OllamaStatusInfo) => void;
  onFolderSelected?: (path: string) => void;
  onIndexProgress?: (progress: IndexProgress) => void;
  onIndexStatsUpdate?: (stats: IndexStats) => void;
  onIndexComplete?: (stats: IndexStats) => void;
  onIndexingCancelled?: () => void;
  onQueryChunk?: (chunk: string) => void;
  onQueryComplete?: (sources: SourceInfo[]) => void;
  onAgentProgress?: (message: string) => void;
  onModelsRefreshed?: (models: ModelsPayload) => void;
  onError?: (error: string) => void;
}) {
  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    const setupListeners = async () => {
      if (callbacks.onOllamaStatus) {
        const unlisten = await listen<boolean>('ollama-status', (event) => {
          callbacks.onOllamaStatus!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onOllamaStatusInfo) {
        const unlisten = await listen<OllamaStatusInfo>('ollama-status-info', (event) => {
          callbacks.onOllamaStatusInfo!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onFolderSelected) {
        const unlisten = await listen<string>('folder-selected', (event) => {
          callbacks.onFolderSelected!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onIndexProgress) {
        const unlisten = await listen<IndexProgress>('index-progress', (event) => {
          callbacks.onIndexProgress!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onIndexStatsUpdate) {
        const unlisten = await listen<IndexStats>('index-stats-update', (event) => {
          callbacks.onIndexStatsUpdate!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onIndexComplete) {
        const unlisten = await listen<IndexStats>('index-complete', (event) => {
          callbacks.onIndexComplete!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onIndexingCancelled) {
        const unlisten = await listen('indexing-cancelled', () => {
          callbacks.onIndexingCancelled!();
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onQueryChunk) {
        const unlisten = await listen<string>('query-chunk', (event) => {
          callbacks.onQueryChunk!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onQueryComplete) {
        const unlisten = await listen<SourceInfo[]>('query-complete', (event) => {
          callbacks.onQueryComplete!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onAgentProgress) {
        const unlisten = await listen<string>('agent-progress', (event) => {
          callbacks.onAgentProgress!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onModelsRefreshed) {
        const unlisten = await listen<ModelsPayload>('models-refreshed', (event) => {
          callbacks.onModelsRefreshed!(event.payload);
        });
        unlisteners.push(unlisten);
      }

      if (callbacks.onError) {
        const unlisten = await listen<string>('error', (event) => {
          callbacks.onError!(event.payload);
        });
        unlisteners.push(unlisten);
      }
    };

    setupListeners();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [callbacks]);

  const selectFolder = useCallback(async (): Promise<string | null> => {
    return await invoke<string | null>('select_folder');
  }, []);

  const startIndexing = useCallback(async (folder: string): Promise<void> => {
    await invoke('start_indexing', { folder });
  }, []);

  const stopIndexing = useCallback(async (): Promise<void> => {
    await invoke('stop_indexing');
  }, []);

  const sendQuery = useCallback(async (question: string, agentMode: boolean): Promise<void> => {
    await invoke('send_query', { question, agentMode });
  }, []);

  const refreshModels = useCallback(async (): Promise<void> => {
    await invoke('refresh_models');
  }, []);

  const setLlmModel = useCallback(async (model: string): Promise<void> => {
    await invoke('set_llm_model', { model });
  }, []);

  const setEmbeddingModel = useCallback(async (model: string): Promise<void> => {
    await invoke('set_embedding_model', { model });
  }, []);

  const getCurrentModels = useCallback(async (): Promise<CurrentModels> => {
    return await invoke<CurrentModels>('get_current_models');
  }, []);

  const checkOllamaStatus = useCallback(async (): Promise<boolean> => {
    return await invoke<boolean>('check_ollama_status');
  }, []);

  const checkOllamaStatusInfo = useCallback(async (): Promise<OllamaStatusInfo> => {
    return await invoke<OllamaStatusInfo>('check_ollama_status_info');
  }, []);

  const getSystemInfo = useCallback(async (): Promise<SystemInfo> => {
    return await invoke<SystemInfo>('get_system_info');
  }, []);

  const getFullSystemStatus = useCallback(async (): Promise<FullSystemStatus> => {
    return await invoke<FullSystemStatus>('get_full_system_status');
  }, []);

  return {
    selectFolder,
    startIndexing,
    stopIndexing,
    sendQuery,
    refreshModels,
    setLlmModel,
    setEmbeddingModel,
    getCurrentModels,
    checkOllamaStatus,
    checkOllamaStatusInfo,
    getSystemInfo,
    getFullSystemStatus,
  };
}
