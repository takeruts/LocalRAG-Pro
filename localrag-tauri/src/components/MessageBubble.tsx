import type { Message } from '../types';

interface MessageBubbleProps {
  message: Message;
  isGenerating: boolean;
  agentMode: boolean;
}

export function MessageBubble({
  message,
  isGenerating,
  agentMode,
}: MessageBubbleProps) {
  if (message.role === 'user') {
    return (
      <div className="flex justify-end">
        <div className="max-w-[80%] bg-user-msg text-text-bright p-3 rounded-xl">
          <p className="text-sm whitespace-pre-wrap">{message.content}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex justify-start">
      <div className="max-w-[80%] bg-assistant-msg text-text-primary p-3 rounded-xl">
        {message.content === '' && isGenerating ? (
          // Typing indicator
          <div className="flex items-center gap-1">
            <div
              className={`w-2 h-2 rounded-full typing-dot ${
                agentMode ? 'bg-warning' : 'bg-primary'
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full typing-dot ${
                agentMode ? 'bg-warning' : 'bg-primary'
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full typing-dot ${
                agentMode ? 'bg-warning' : 'bg-primary'
              }`}
            />
          </div>
        ) : (
          <>
            <p className="text-sm whitespace-pre-wrap">{message.content}</p>
            {isGenerating && message.content && (
              <span className="inline-block w-2 h-4 bg-primary animate-pulse ml-1" />
            )}
          </>
        )}
      </div>
    </div>
  );
}
