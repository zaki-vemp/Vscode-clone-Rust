import React from 'react';
import {
  GitBranch,
  RefreshCw,
  AlertCircle,
  AlertTriangle,
  Bell,
  Cpu,
  Radio,
  Sparkles,
} from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';
import { EditorSettings } from '../types/editor.ts';

interface StatusBarProps {
  theme: ThemeColors;
  settings: EditorSettings;
  activeLanguage: string;
  cursorLine: number;
  cursorCol: number;
  errorsCount: number;
  warningsCount: number;
  currentLatencyMs: number;
  onOpenRustProfiler: () => void;
  onOpenCommandPalette: () => void;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  theme,
  settings,
  activeLanguage,
  cursorLine,
  cursorCol,
  errorsCount,
  warningsCount,
  currentLatencyMs,
  onOpenRustProfiler,
  onOpenCommandPalette,
}) => {
  return (
    <footer
      className="h-6 w-full flex items-center justify-between px-2 text-[11px] select-none shrink-0 font-sans z-30"
      style={{
        backgroundColor: theme.statusBarBg,
        color: theme.statusBarFg,
      }}
      aria-label="Status Bar"
    >
      {/* Left items */}
      <div className="flex items-center space-x-3">
        {/* Branch */}
        <button
          type="button"
          onClick={onOpenCommandPalette}
          className="flex items-center gap-1 hover:opacity-80 transition-opacity"
          title="Git Branch: main"
        >
          <GitBranch className="w-3 h-3" />
          <span className="font-medium">main*</span>
        </button>

        {/* Sync */}
        <button
          type="button"
          className="hover:opacity-80 transition-opacity"
          title="Synchronize Changes"
        >
          <RefreshCw className="w-3 h-3" />
        </button>

        {/* Problems */}
        <div className="flex items-center gap-2">
          <span className="flex items-center gap-0.5" title="Errors">
            <AlertCircle className="w-3 h-3" />
            <span>{errorsCount}</span>
          </span>
          <span className="flex items-center gap-0.5" title="Warnings">
            <AlertTriangle className="w-3 h-3" />
            <span>{warningsCount}</span>
          </span>
        </div>

        {/* rust-analyzer LSP status */}
        <div
          className="flex items-center gap-1 px-1.5 py-0.2 rounded bg-black/10 text-[10px] font-mono cursor-pointer hover:bg-black/20"
          title="rust-analyzer Language Server (v0.3.2100): Idle"
          onClick={onOpenRustProfiler}
        >
          <span>🦀</span>
          <span>rust-analyzer: idle</span>
          <span className="opacity-75">0.4ms</span>
        </div>

        {/* Copilot status */}
        <div
          className="flex items-center gap-1 text-[10px] opacity-90 cursor-pointer hover:opacity-100"
          title="GitHub Copilot Native: Ready"
        >
          <Sparkles className="w-3 h-3 text-amber-200" />
          <span>Copilot</span>
        </div>
      </div>

      {/* Right items */}
      <div className="flex items-center space-x-3">
        {/* Latency Live Meter */}
        <button
          type="button"
          onClick={onOpenRustProfiler}
          className="flex items-center gap-1 font-mono text-[10px] px-1.5 py-0.2 rounded bg-black/20 hover:bg-black/30 transition-colors"
          title="Rust Native Typing Latency"
        >
          <span className="text-amber-200 font-bold">⚡ {currentLatencyMs}ms</span>
          <span className="opacity-75">· 22MB RAM</span>
        </button>

        {/* Cursor Position */}
        <span className="font-mono">
          Ln {cursorLine}, Col {cursorCol}
        </span>

        {/* Indentation */}
        <span>Spaces: {settings.tabSize}</span>

        {/* Encoding */}
        <span>UTF-8</span>

        {/* Line Endings */}
        <span>LF</span>

        {/* Language Mode */}
        <button
          type="button"
          onClick={onOpenCommandPalette}
          className="font-medium hover:underline capitalize"
          title="Select Language Mode"
        >
          {activeLanguage === 'rust' ? 'Rust (🦀)' : activeLanguage}
        </button>

        {/* Notifications */}
        <button
          type="button"
          className="hover:opacity-80 transition-opacity"
          title="Notifications"
        >
          <Bell className="w-3 h-3" />
        </button>
      </div>
    </footer>
  );
};
