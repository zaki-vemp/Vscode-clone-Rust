import React, { useState, useRef, useEffect } from 'react';
import {
  Terminal as TerminalIcon,
  AlertTriangle,
  FileCode,
  Bug,
  Activity,
  Maximize2,
  Minimize2,
  X,
  Trash2,
  Plus,
  ArrowRight,
  Lightbulb,
} from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';
import { executeCommand, TerminalOutputLine } from '../logic/terminalEngine.ts';
import { Diagnostic } from '../types/editor.ts';

export type BottomTab = 'terminal' | 'problems' | 'output' | 'debug' | 'engine';

interface BottomPanelProps {
  theme: ThemeColors;
  isOpen: boolean;
  onClose: () => void;
  diagnostics: Diagnostic[];
  onSelectDiagnostic: (d: Diagnostic) => void;
  onRunQuickFix: (d: Diagnostic) => void;
  terminalCommandToRun?: string;
  onClearRunCommand?: () => void;
}

export const BottomPanel: React.FC<BottomPanelProps> = ({
  theme,
  isOpen,
  onClose,
  diagnostics,
  onSelectDiagnostic,
  onRunQuickFix,
  terminalCommandToRun,
  onClearRunCommand,
}) => {
  const [activeTab, setActiveTab] = useState<BottomTab>('terminal');
  const [isMaximized, setIsMaximized] = useState(false);
  const [inputVal, setInputVal] = useState('');
  const [terminalLines, setTerminalLines] = useState<TerminalOutputLine[]>([
    {
      id: 'init-1',
      type: 'info',
      text: '🦀 OxideCode Rust Terminal Host v0.1.0 (wgpu native engine)',
    },
    {
      id: 'init-2',
      type: 'success',
      text: '✓ Rust toolchain initialized: rustc 1.82.0 (x86_64-unknown-linux-gnu)',
    },
    {
      id: 'init-3',
      type: 'output',
      text: 'Type "help" for cargo commands, or run "bench" to compare Rust vs Electron.',
    },
  ]);

  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);
  const terminalScrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll terminal on new lines
  useEffect(() => {
    if (terminalScrollRef.current) {
      terminalScrollRef.current.scrollTop = terminalScrollRef.current.scrollHeight;
    }
  }, [terminalLines]);

  // Execute terminal command triggered externally (e.g. from TitleBar / SideBar)
  useEffect(() => {
    if (terminalCommandToRun) {
      runCommand(terminalCommandToRun);
      if (onClearRunCommand) onClearRunCommand();
    }
  }, [terminalCommandToRun]);

  const runCommand = (cmdText: string) => {
    const trimmed = cmdText.trim();
    if (!trimmed) return;

    // Add user command line
    const userLine: TerminalOutputLine = {
      id: Math.random().toString(),
      type: 'input',
      text: `[oxide-editor] ~/dev/oxide-editor (main) $ ${trimmed}`,
    };

    const result = executeCommand(trimmed);

    if (result.clear) {
      setTerminalLines([]);
    } else {
      setTerminalLines((prev) => [...prev, userLine, ...result.lines]);
    }

    setHistory((prev) => [...prev, trimmed]);
    setHistoryIdx(-1);
    setInputVal('');
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      runCommand(inputVal);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (history.length > 0) {
        const nextIdx = historyIdx === -1 ? history.length - 1 : Math.max(0, historyIdx - 1);
        setHistoryIdx(nextIdx);
        setInputVal(history[nextIdx] || '');
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIdx !== -1) {
        const nextIdx = historyIdx + 1;
        if (nextIdx >= history.length) {
          setHistoryIdx(-1);
          setInputVal('');
        } else {
          setHistoryIdx(nextIdx);
          setInputVal(history[nextIdx]);
        }
      }
    }
  };

  if (!isOpen) return null;

  return (
    <section
      className={`border-t flex flex-col select-none transition-all duration-150 relative z-10 ${
        isMaximized ? 'h-[85vh]' : 'h-64'
      }`}
      style={{
        backgroundColor: theme.panelBg,
        borderColor: theme.border,
        color: theme.textPrimary,
      }}
      aria-label="Integrated Panel"
    >
      {/* Panel Tab Header */}
      <div
        className="h-8 px-3 flex items-center justify-between border-b shrink-0 text-xs"
        style={{ borderColor: theme.border }}
      >
        {/* Left Tabs */}
        <div className="flex items-center space-x-1">
          <button
            type="button"
            onClick={() => setActiveTab('terminal')}
            className={`px-3 py-1 font-semibold flex items-center gap-1.5 rounded transition-colors text-[11px] ${
              activeTab === 'terminal'
                ? 'text-white border-b-2 border-orange-500'
                : 'text-zinc-400 hover:text-white'
            }`}
          >
            <TerminalIcon className="w-3.5 h-3.5" />
            <span>TERMINAL</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('problems')}
            className={`px-3 py-1 font-semibold flex items-center gap-1.5 rounded transition-colors text-[11px] ${
              activeTab === 'problems'
                ? 'text-white border-b-2 border-orange-500'
                : 'text-zinc-400 hover:text-white'
            }`}
          >
            <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
            <span>PROBLEMS</span>
            <span className="px-1.5 py-0.2 rounded-full bg-zinc-800 text-[10px] text-zinc-300">
              {diagnostics.length}
            </span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('output')}
            className={`px-3 py-1 font-semibold flex items-center gap-1.5 rounded transition-colors text-[11px] ${
              activeTab === 'output'
                ? 'text-white border-b-2 border-orange-500'
                : 'text-zinc-400 hover:text-white'
            }`}
          >
            <FileCode className="w-3.5 h-3.5" />
            <span>OUTPUT (rust-analyzer)</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('debug')}
            className={`px-3 py-1 font-semibold flex items-center gap-1.5 rounded transition-colors text-[11px] ${
              activeTab === 'debug'
                ? 'text-white border-b-2 border-orange-500'
                : 'text-zinc-400 hover:text-white'
            }`}
          >
            <Bug className="w-3.5 h-3.5" />
            <span>DEBUG CONSOLE</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('engine')}
            className={`px-3 py-1 font-semibold flex items-center gap-1.5 rounded transition-colors text-[11px] ${
              activeTab === 'engine'
                ? 'text-white border-b-2 border-orange-500'
                : 'text-orange-400 hover:text-orange-300'
            }`}
          >
            <Activity className="w-3.5 h-3.5" />
            <span>RUST TELEMETRY</span>
          </button>
        </div>

        {/* Right Tab Controls */}
        <div className="flex items-center space-x-1 text-zinc-400">
          {activeTab === 'terminal' && (
            <>
              <button
                type="button"
                onClick={() => setTerminalLines([])}
                className="p-1 hover:text-white rounded hover:bg-zinc-800"
                title="Clear Terminal"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => runCommand('cargo check')}
                className="p-1 hover:text-white rounded hover:bg-zinc-800"
                title="Run cargo check"
              >
                <Plus className="w-3.5 h-3.5" />
              </button>
            </>
          )}

          <button
            type="button"
            onClick={() => setIsMaximized(!isMaximized)}
            className="p-1 hover:text-white rounded hover:bg-zinc-800"
            title={isMaximized ? 'Restore Panel Size' : 'Maximize Panel'}
          >
            {isMaximized ? (
              <Minimize2 className="w-3.5 h-3.5" />
            ) : (
              <Maximize2 className="w-3.5 h-3.5" />
            )}
          </button>

          <button
            type="button"
            onClick={onClose}
            className="p-1 hover:text-white rounded hover:bg-zinc-800"
            title="Close Panel (Ctrl+J)"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Tab Content Body */}
      <div className="flex-1 overflow-hidden relative">
        {/* ================= TERMINAL TAB ================= */}
        {activeTab === 'terminal' && (
          <div className="h-full flex flex-col font-mono text-xs p-3">
            {/* Scrollable output */}
            <div
              ref={terminalScrollRef}
              className="flex-1 overflow-y-auto space-y-1 select-text"
            >
              {terminalLines.map((line) => {
                let colorClass = 'text-zinc-300';
                if (line.type === 'input') colorClass = 'text-orange-400 font-semibold';
                if (line.type === 'success') colorClass = 'text-emerald-400 font-medium';
                if (line.type === 'error') colorClass = 'text-red-400';
                if (line.type === 'info') colorClass = 'text-sky-400';

                return (
                  <div key={line.id} className={`whitespace-pre-wrap ${colorClass}`}>
                    {line.text}
                  </div>
                );
              })}
            </div>

            {/* Input Prompt */}
            <div className="flex items-center gap-2 pt-2 border-t border-zinc-800/80 mt-1">
              <span className="text-orange-500 font-bold select-none text-[11px]">
                [oxide-editor] ~/dev/oxide-editor (main) $
              </span>
              <input
                type="text"
                value={inputVal}
                onChange={(e) => setInputVal(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="type 'cargo run', 'cargo test', 'bench', or 'help'..."
                className="flex-1 bg-transparent text-white outline-none text-xs font-mono placeholder-zinc-600"
                autoFocus
              />
            </div>
          </div>
        )}

        {/* ================= PROBLEMS TAB ================= */}
        {activeTab === 'problems' && (
          <div className="h-full overflow-y-auto p-3 space-y-2 text-xs">
            <div className="text-[11px] text-zinc-400 font-semibold uppercase tracking-wider">
              {diagnostics.length} PROBLEMS DETECTED BY CLIPPY &amp; RUST-ANALYZER
            </div>

            {diagnostics.map((diag) => (
              <div
                key={diag.id}
                onClick={() => onSelectDiagnostic(diag)}
                className="p-2.5 rounded bg-zinc-900 border border-zinc-800 hover:border-orange-500/50 cursor-pointer flex items-start justify-between transition-colors group"
              >
                <div className="flex items-start gap-2">
                  <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
                  <div>
                    <div className="font-semibold text-zinc-200">{diag.message}</div>
                    <div className="text-[11px] text-zinc-500 font-mono mt-0.5">
                      {diag.filePath}:{diag.line}:{diag.column} · source: {diag.source}
                    </div>
                  </div>
                </div>

                {diag.quickFix && (
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      onRunQuickFix(diag);
                    }}
                    className="px-2 py-1 rounded bg-orange-600 hover:bg-orange-500 text-white font-medium text-[10px] flex items-center gap-1 transition-colors"
                  >
                    <Lightbulb className="w-3 h-3" />
                    <span>Quick Fix</span>
                  </button>
                )}
              </div>
            ))}
          </div>
        )}

        {/* ================= OUTPUT TAB ================= */}
        {activeTab === 'output' && (
          <div className="h-full overflow-y-auto p-3 font-mono text-[11px] text-zinc-400 space-y-1 select-text">
            <div className="text-zinc-500">// Language Server Protocol (rust-analyzer JSON-RPC)</div>
            <div>[rust-analyzer] Server PID: 49120 · Target: x86_64-unknown-linux-gnu</div>
            <div>[rust-analyzer] Loaded workspace with 4 crates in 14.2ms</div>
            <div className="text-emerald-400">[rust-analyzer] PublishDiagnostics: 0 errors, 1 warning (0.4ms)</div>
            <div>[extension-host] WebAssembly isolation enabled. Memory limit: 256MB</div>
            <div>[extension-host] Activated extensions: rust-analyzer, gitlens, prettier, copilot</div>
          </div>
        )}

        {/* ================= DEBUG CONSOLE TAB ================= */}
        {activeTab === 'debug' && (
          <div className="h-full overflow-y-auto p-3 font-mono text-xs text-zinc-400 space-y-2">
            <div className="text-zinc-500">// LLDB Rust Native Debugger Expression Evaluator</div>
            <div className="text-zinc-300">&gt; buffer.line_count()</div>
            <div className="text-emerald-400">=&gt; 46</div>
            <div className="text-zinc-300">&gt; wgpu_surface.fps()</div>
            <div className="text-emerald-400">=&gt; 120.0 FPS</div>
          </div>
        )}

        {/* ================= RUST TELEMETRY TAB ================= */}
        {activeTab === 'engine' && (
          <div className="h-full overflow-y-auto p-3 text-xs space-y-3">
            <div className="grid grid-cols-4 gap-3">
              <div className="p-3 rounded bg-zinc-900 border border-zinc-800">
                <div className="text-zinc-400 text-[10px] uppercase font-bold">Key-to-Glass Latency</div>
                <div className="text-xl font-bold font-mono text-orange-400">0.8 ms</div>
                <div className="text-[10px] text-emerald-400">⚡ 30x faster than Electron</div>
              </div>
              <div className="p-3 rounded bg-zinc-900 border border-zinc-800">
                <div className="text-zinc-400 text-[10px] uppercase font-bold">Heap Memory (mimalloc)</div>
                <div className="text-xl font-bold font-mono text-emerald-400">22.4 MB</div>
                <div className="text-[10px] text-zinc-400">No V8 GC pauses</div>
              </div>
              <div className="p-3 rounded bg-zinc-900 border border-zinc-800">
                <div className="text-zinc-400 text-[10px] uppercase font-bold">Render Target</div>
                <div className="text-xl font-bold font-mono text-purple-400">wgpu Native</div>
                <div className="text-[10px] text-zinc-400">120 FPS Glyph Cache</div>
              </div>
              <div className="p-3 rounded bg-zinc-900 border border-zinc-800">
                <div className="text-zinc-400 text-[10px] uppercase font-bold">Extension Host</div>
                <div className="text-xl font-bold font-mono text-sky-400">Wasmtime</div>
                <div className="text-[10px] text-zinc-400">Pure Sandboxed WASI</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </section>
  );
};
