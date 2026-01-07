import { useState } from 'react';
import type { SourceInfo } from '../types';

interface SourceInfoPanelProps {
  sources: SourceInfo[];
}

export function SourceInfoPanel({ sources }: SourceInfoPanelProps) {
  const [isOpen, setIsOpen] = useState(false);

  if (sources.length === 0) return null;

  return (
    <div className="bg-bg-card rounded-lg overflow-hidden">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full p-3 flex items-center justify-between text-sm text-text-secondary hover:bg-bg-input transition-colors"
      >
        <span>Source Information ({sources.length})</span>
        <span className={`transform transition-transform ${isOpen ? 'rotate-180' : ''}`}>
          ▼
        </span>
      </button>

      {isOpen && (
        <div className="p-3 border-t border-bg-main flex flex-col gap-2">
          {sources.map((source, index) => (
            <div key={index} className="flex flex-col gap-1">
              <div className="flex items-start gap-2">
                <span className="text-primary text-xs font-medium">{index + 1}.</span>
                <div className="flex-1">
                  <p className="text-xs text-text-primary">
                    {source.source}
                    {source.page !== null && (
                      <span className="text-text-muted ml-1">(P.{source.page + 1})</span>
                    )}
                  </p>
                  <p className="text-xs text-text-muted">
                    Score: {source.score.toFixed(2)}
                  </p>
                </div>
              </div>
              {index < sources.length - 1 && (
                <div className="border-b border-bg-main my-1" />
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
