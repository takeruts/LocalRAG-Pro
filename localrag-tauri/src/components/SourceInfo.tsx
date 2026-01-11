import { useState } from 'react';
import type { SourceInfo } from '../types';

interface SourceInfoPanelProps {
  sources: SourceInfo[];
}

// Extract file name and folder from full path
function parseSourcePath(sourcePath: string): { fileName: string; folder: string } {
  if (!sourcePath) {
    return { fileName: 'Unknown', folder: '' };
  }

  // Normalize path separators
  const normalizedPath = sourcePath.replace(/\\/g, '/');
  const parts = normalizedPath.split('/');
  const fileName = parts[parts.length - 1] || 'Unknown';
  const folder = parts.slice(0, -1).join('/') || '';

  return { fileName, folder };
}

export function SourceInfoPanel({ sources }: SourceInfoPanelProps) {
  const [isOpen, setIsOpen] = useState(true); // Default to open

  if (sources.length === 0) return null;

  return (
    <div className="bg-bg-card rounded-lg overflow-hidden">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full p-3 flex items-center justify-between text-sm text-text-secondary hover:bg-bg-input transition-colors"
      >
        <span>Related Documents ({sources.length})</span>
        <span className={`transform transition-transform ${isOpen ? 'rotate-180' : ''}`}>
          ▼
        </span>
      </button>

      {isOpen && (
        <div className="p-3 border-t border-bg-main flex flex-col gap-2 max-h-80 overflow-y-auto">
          {sources.map((source, index) => {
            const { fileName, folder } = parseSourcePath(source.source);
            return (
              <div key={index} className="flex flex-col gap-1 bg-bg-main p-2 rounded">
                <div className="flex items-start gap-2">
                  <span className="text-primary text-xs font-bold min-w-[20px]">{index + 1}.</span>
                  <div className="flex-1 min-w-0">
                    {/* File name with page */}
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="text-sm text-text-primary font-medium truncate">
                        {fileName}
                      </span>
                      {source.page !== null && (
                        <span className="text-xs text-warning bg-warning/20 px-1.5 py-0.5 rounded">
                          Page {source.page + 1}
                        </span>
                      )}
                      <span className="text-xs text-success bg-success/20 px-1.5 py-0.5 rounded">
                        {(source.score * 100).toFixed(0)}%
                      </span>
                    </div>
                    {/* Folder path */}
                    {folder && (
                      <p className="text-xs text-text-muted mt-1 truncate" title={folder}>
                        {folder}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
