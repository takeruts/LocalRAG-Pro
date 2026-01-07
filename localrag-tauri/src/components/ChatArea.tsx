import { useState, useRef, useEffect } from 'react';
import type { Message, SourceInfo } from '../types';
import { MessageBubble } from './MessageBubble';
import { SourceInfoPanel } from './SourceInfo';

interface ChatAreaProps {
  messages: Message[];
  isGenerating: boolean;
  agentMode: boolean;
  agentProgress: string;
  currentSources: SourceInfo[];
  onSendMessage: (message: string) => void;
  onToggleAgentMode: () => void;
}

export function ChatArea({
  messages,
  isGenerating,
  agentMode,
  agentProgress,
  currentSources,
  onSendMessage,
  onToggleAgentMode,
}: ChatAreaProps) {
  const [inputText, setInputText] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, agentProgress]);

  const handleSubmit = () => {
    if (inputText.trim() && !isGenerating) {
      onSendMessage(inputText.trim());
      setInputText('');
    }
  };

  const handleKeyDown = (_e: React.KeyboardEvent) => {
    // Don't submit on Enter to avoid IME issues
    // Users should click the send button
  };

  return (
    <div className="flex-1 flex flex-col bg-bg-main">
      {/* Messages Area */}
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto p-4 flex flex-col gap-3"
      >
        {messages.map((msg, index) => (
          <MessageBubble
            key={index}
            message={msg}
            isGenerating={isGenerating && index === messages.length - 1 && msg.role === 'assistant'}
            agentMode={agentMode}
          />
        ))}

        {/* Agent Progress */}
        {agentMode && agentProgress && (
          <div className="bg-bg-card p-3 rounded-lg">
            <div className="flex items-center gap-2">
              <div className="w-4 h-4 border-2 border-warning border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-warning">{agentProgress}</span>
            </div>
          </div>
        )}

        {/* Thinking Indicator (non-agent mode) */}
        {!agentMode && isGenerating && messages.length > 0 && messages[messages.length - 1].content === '' && (
          <div className="bg-bg-card p-3 rounded-lg">
            <div className="flex items-center gap-2">
              <span className="text-sm text-primary animate-pulse">Searching...</span>
            </div>
          </div>
        )}

        {/* Sources */}
        {currentSources.length > 0 && (
          <SourceInfoPanel sources={currentSources} />
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="border-t border-bg-card p-4">
        {/* Mode Indicator */}
        <div className="flex items-center gap-2 mb-2 text-xs">
          <span className="text-text-muted">Mode:</span>
          <span className={agentMode ? 'text-warning' : 'text-primary'}>
            {agentMode ? 'Agent (Auto Search)' : 'RAG (Simple Search)'}
          </span>
          <span className="text-text-muted">← Click to switch</span>
        </div>

        <div className="flex gap-2">
          {/* Mode Toggle Button */}
          <button
            onClick={onToggleAgentMode}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              agentMode
                ? 'bg-warning text-text-bright'
                : 'bg-primary-dim text-text-bright'
            }`}
          >
            {agentMode ? 'Agent' : 'RAG'}
          </button>

          {/* Input Field */}
          <input
            type="text"
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Enter your question..."
            className="flex-1 bg-bg-card text-text-primary p-3 rounded-lg text-sm border-none outline-none placeholder:text-text-muted"
            disabled={isGenerating}
          />

          {/* Send Button */}
          <button
            onClick={handleSubmit}
            disabled={!inputText.trim() || isGenerating}
            className={`px-6 py-2 rounded-lg text-sm font-medium transition-colors ${
              !inputText.trim() || isGenerating
                ? 'bg-bg-card text-text-muted cursor-not-allowed'
                : 'bg-success text-text-bright hover:opacity-90'
            }`}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}
