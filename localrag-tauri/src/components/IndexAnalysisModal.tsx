import { useState } from 'react';
import { open } from '@tauri-apps/plugin-shell';
import type { IndexAnalysis } from '../types';

interface IndexAnalysisModalProps {
  analysis: IndexAnalysis;
  onClose: () => void;
}

type ViewMode = 'folders' | 'files';

export function IndexAnalysisModal({ analysis, onClose }: IndexAnalysisModalProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('folders');
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());

  const handleOpenFolder = async (folderPath: string) => {
    try {
      await open(folderPath);
    } catch (error) {
      console.error('Failed to open folder:', error);
    }
  };

  const handleOpenFile = async (filePath: string) => {
    try {
      await open(filePath);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const toggleFolder = (folder: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folder)) {
      newExpanded.delete(folder);
    } else {
      newExpanded.add(folder);
    }
    setExpandedFolders(newExpanded);
  };

  const getFilesInFolder = (folder: string) => {
    return analysis.files.filter((file) => {
      const normalizedPath = file.path.replace(/\\/g, '/');
      const normalizedFolder = folder.replace(/\\/g, '/');
      return normalizedPath.startsWith(normalizedFolder + '/') ||
             normalizedPath.startsWith(normalizedFolder + '\\');
    });
  };

  const getFileName = (path: string) => {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1];
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-bg-card rounded-lg w-[700px] max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-bg-main flex items-center justify-between">
          <h2 className="text-lg font-bold text-text-primary">DB Analysis</h2>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text-primary transition-colors text-xl"
          >
            ×
          </button>
        </div>

        {/* Summary */}
        <div className="p-4 border-b border-bg-main">
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div className="bg-bg-main p-3 rounded-lg text-center">
              <p className="text-text-muted">Total Files</p>
              <p className="text-2xl font-bold text-primary">{analysis.total_files}</p>
            </div>
            <div className="bg-bg-main p-3 rounded-lg text-center">
              <p className="text-text-muted">Total Chunks</p>
              <p className="text-2xl font-bold text-primary">{analysis.total_chunks}</p>
            </div>
            <div className="bg-bg-main p-3 rounded-lg text-center">
              <p className="text-text-muted">Folders</p>
              <p className="text-2xl font-bold text-primary">{analysis.folders.length}</p>
            </div>
          </div>
        </div>

        {/* View Toggle */}
        <div className="p-4 border-b border-bg-main">
          <div className="flex gap-2">
            <button
              onClick={() => setViewMode('folders')}
              className={`px-4 py-2 rounded-lg text-sm transition-colors ${
                viewMode === 'folders'
                  ? 'bg-primary text-text-bright'
                  : 'bg-bg-main text-text-secondary hover:bg-bg-input'
              }`}
            >
              Folders
            </button>
            <button
              onClick={() => setViewMode('files')}
              className={`px-4 py-2 rounded-lg text-sm transition-colors ${
                viewMode === 'files'
                  ? 'bg-primary text-text-bright'
                  : 'bg-bg-main text-text-secondary hover:bg-bg-input'
              }`}
            >
              Files
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {viewMode === 'folders' ? (
            <div className="flex flex-col gap-2">
              {analysis.folders.map((folder, index) => (
                <div key={index} className="bg-bg-main rounded-lg overflow-hidden">
                  <div
                    className="p-3 flex items-center justify-between cursor-pointer hover:bg-bg-input transition-colors"
                    onClick={() => toggleFolder(folder.folder)}
                  >
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                      <span className={`transform transition-transform ${expandedFolders.has(folder.folder) ? 'rotate-90' : ''}`}>
                        ▶
                      </span>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleOpenFolder(folder.folder);
                        }}
                        className="text-sm text-primary truncate hover:underline text-left"
                        title={folder.folder}
                      >
                        {folder.folder}
                      </button>
                    </div>
                    <div className="flex gap-4 text-xs text-text-muted flex-shrink-0">
                      <span>{folder.file_count} files</span>
                      <span>{folder.chunk_count} chunks</span>
                    </div>
                  </div>
                  {expandedFolders.has(folder.folder) && (
                    <div className="border-t border-bg-card px-3 py-2">
                      {getFilesInFolder(folder.folder).map((file, fileIndex) => (
                        <div
                          key={fileIndex}
                          className="flex items-center justify-between py-1 text-sm"
                        >
                          <button
                            onClick={() => handleOpenFile(file.path)}
                            className="text-text-secondary hover:text-primary hover:underline truncate text-left"
                            title={file.path}
                          >
                            {getFileName(file.path)}
                          </button>
                          <span className="text-xs text-text-muted flex-shrink-0">
                            {file.chunk_count} chunks
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="flex flex-col gap-1">
              {analysis.files.map((file, index) => (
                <div
                  key={index}
                  className="bg-bg-main p-2 rounded flex items-center justify-between"
                >
                  <button
                    onClick={() => handleOpenFile(file.path)}
                    className="text-sm text-text-secondary hover:text-primary hover:underline truncate text-left flex-1"
                    title={file.path}
                  >
                    {file.path}
                  </button>
                  <span className="text-xs text-text-muted flex-shrink-0 ml-2">
                    {file.chunk_count} chunks
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-bg-main">
          <button
            onClick={onClose}
            className="w-full py-2 bg-bg-main text-text-secondary rounded-lg hover:bg-bg-input transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
