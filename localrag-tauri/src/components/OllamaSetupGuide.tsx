import type { SystemInfo } from '../types';

interface OllamaSetupGuideProps {
  onClose: () => void;
  systemInfo?: SystemInfo | null;
  ollamaRunning?: boolean | null;
}

export function OllamaSetupGuide({
  onClose,
  systemInfo,
  ollamaRunning: _ollamaRunning
}: OllamaSetupGuideProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-bg-card rounded-xl p-6 max-w-lg w-full mx-4 shadow-2xl max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 bg-warning/20 rounded-full flex items-center justify-center">
            <span className="text-xl">i</span>
          </div>
          <div>
            <h2 className="text-lg font-bold text-text-bright">Ollama Setup</h2>
            <p className="text-sm text-text-muted">System information & setup guide</p>
          </div>
        </div>

        {/* System Info Section */}
        {systemInfo && (
          <div className="bg-bg-main rounded-lg p-4 mb-4">
            <h3 className="text-sm font-medium text-text-primary mb-3">
              System Information
            </h3>
            <div className="space-y-2 text-xs">
              {systemInfo.cpu_name && (
                <div className="flex items-start gap-2">
                  <span className="text-text-muted w-16 flex-shrink-0">CPU:</span>
                  <span className="text-text-primary">{systemInfo.cpu_name}</span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Content */}
        <div className="space-y-4 mb-6">
          <p className="text-sm text-text-secondary">
            CPURAG requires Ollama to run local AI models. Please follow the steps below to install and configure Ollama.
          </p>

          {/* Step 1 */}
          <div className="bg-bg-main rounded-lg p-4">
            <div className="flex items-start gap-3">
              <span className="w-6 h-6 bg-primary rounded-full flex items-center justify-center text-xs font-bold text-white flex-shrink-0">
                1
              </span>
              <div>
                <h3 className="font-medium text-text-primary mb-1">Install Ollama</h3>
                <p className="text-sm text-text-muted">
                  This application requires Ollama to run local AI models. Please ensure Ollama is installed on your system before proceeding.
                </p>
              </div>
            </div>
          </div>

          {/* Step 2 */}
          <div className="bg-bg-main rounded-lg p-4">
            <div className="flex items-start gap-3">
              <span className="w-6 h-6 bg-primary rounded-full flex items-center justify-center text-xs font-bold text-white flex-shrink-0">
                2
              </span>
              <div>
                <h3 className="font-medium text-text-primary mb-1">Install Required Models</h3>
                <p className="text-sm text-text-muted mb-2">
                  After installation, open a terminal and run these commands:
                </p>
                <div className="bg-bg-input rounded-lg p-3 font-mono text-xs text-text-primary space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-text-muted">$</span>
                    <code>ollama pull gemma3:4b</code>
                    <span className="text-text-muted ml-auto"># LLM model</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-text-muted">$</span>
                    <code>ollama pull bge-m3</code>
                    <span className="text-text-muted ml-auto"># Embedding</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Step 3 */}
          <div className="bg-bg-main rounded-lg p-4">
            <div className="flex items-start gap-3">
              <span className="w-6 h-6 bg-primary rounded-full flex items-center justify-center text-xs font-bold text-white flex-shrink-0">
                3
              </span>
              <div>
                <h3 className="font-medium text-text-primary mb-1">Start Ollama</h3>
                <p className="text-sm text-text-muted mb-2">
                  Make sure Ollama is running. On Windows, it should start automatically.
                  Otherwise, run:
                </p>
                <div className="bg-bg-input rounded-lg p-3 font-mono text-xs text-text-primary">
                  <div className="flex items-center gap-2">
                    <span className="text-text-muted">$</span>
                    <code>ollama serve</code>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* How to Use Section */}
          <div className="border-t border-bg-input pt-4 mt-2">
            <h3 className="text-sm font-medium text-text-primary mb-3">
              How to Use CPURAG
            </h3>
            <div className="space-y-3 text-xs text-text-muted">
              <div className="flex items-start gap-2">
                <span className="text-primary font-bold">1.</span>
                <div>
                  <span className="text-text-secondary font-medium">Select a folder</span>
                  <p>Click "Click to select folder..." to choose a folder containing your documents (PDF, TXT, MD, DOCX, etc.)</p>
                </div>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary font-bold">2.</span>
                <div>
                  <span className="text-text-secondary font-medium">Index documents</span>
                  <p>Click "Start Indexing" to process and embed your documents. This may take a while depending on the number of files.</p>
                </div>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary font-bold">3.</span>
                <div>
                  <span className="text-text-secondary font-medium">Ask questions</span>
                  <p>Type your question in the chat area and press Enter. The AI will search your documents and provide answers with sources.</p>
                </div>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary font-bold">4.</span>
                <div>
                  <span className="text-text-secondary font-medium">Agent Mode (Optional)</span>
                  <p>Toggle "Agent Mode" for multi-step reasoning. The AI will break down complex questions and search multiple times.</p>
                </div>
              </div>
            </div>
          </div>

          {/* Tips Section */}
          <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-3">
            <div className="flex items-start gap-2">
              <div className="text-xs text-green-300">
                <span className="font-medium">Tips:</span>
                <ul className="mt-1 space-y-1 list-disc list-inside">
                  <li>Keep Ollama running in the background for best performance</li>
                  <li>Use smaller models (like gemma3:4b) for faster responses</li>
                  <li>Re-index when you add new documents to the folder</li>
                </ul>
              </div>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-bg-input text-text-secondary rounded-lg text-sm hover:bg-bg-main transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
