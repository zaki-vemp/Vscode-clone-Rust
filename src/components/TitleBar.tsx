import React, { useState } from 'react';
import {
  Search,
  PanelLeft,
  PanelBottom,
  Columns,
  Cpu,
  ChevronDown,
  Check,
} from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';
import { EditorSettings } from '../types/editor.ts';

interface TitleBarProps {
  theme: ThemeColors;
  settings: EditorSettings;
  activeFileName: string;
  onOpenCommandPalette: () => void;
  onOpenQuickOpen: () => void;
  onToggleSidebar: () => void;
  onToggleBottomPanel: () => void;
  onToggleSplitEditor: () => void;
  onOpenRustProfiler: () => void;
  onExecuteCommand: (cmd: string) => void;
}

export const TitleBar: React.FC<TitleBarProps> = ({
  theme,
  settings,
  activeFileName,
  onOpenCommandPalette,
  onOpenQuickOpen,
  onToggleSidebar,
  onToggleBottomPanel,
  onToggleSplitEditor,
  onOpenRustProfiler,
  onExecuteCommand,
}) => {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);

  const menus: Record<string, { label: string; action: () => void; shortcut?: string; divider?: boolean }[]> = {
    File: [
      { label: 'New File', action: () => onOpenCommandPalette(), shortcut: 'Ctrl+N' },
      { label: 'Save', action: () => onExecuteCommand('cargo check'), shortcut: 'Ctrl+S' },
      { label: 'Format Document (rustfmt)', action: () => onExecuteCommand('format'), shortcut: 'Shift+Alt+F' },
      { label: 'Close Editor', action: () => {}, shortcut: 'Ctrl+W', divider: true },
      { label: 'Exit OxideCode', action: () => {}, shortcut: 'Ctrl+Q' },
    ],
    Edit: [
      { label: 'Undo', action: () => {}, shortcut: 'Ctrl+Z' },
      { label: 'Redo', action: () => {}, shortcut: 'Ctrl+Y' },
      { label: 'Cut', action: () => {}, shortcut: 'Ctrl+X' },
      { label: 'Copy', action: () => {}, shortcut: 'Ctrl+C' },
      { label: 'Paste', action: () => {}, shortcut: 'Ctrl+V', divider: true },
      { label: 'Find in Files', action: onOpenCommandPalette, shortcut: 'Ctrl+Shift+F' },
    ],
    Selection: [
      { label: 'Select All', action: () => {}, shortcut: 'Ctrl+A' },
      { label: 'Expand Selection', action: () => {}, shortcut: 'Shift+Alt+Right' },
      { label: 'Add Cursor Above', action: () => {}, shortcut: 'Ctrl+Alt+Up' },
      { label: 'Add Cursor Below', action: () => {}, shortcut: 'Ctrl+Alt+Down' },
    ],
    View: [
      { label: 'Command Palette...', action: onOpenCommandPalette, shortcut: 'Ctrl+Shift+P' },
      { label: 'Toggle Primary Side Bar', action: onToggleSidebar, shortcut: 'Ctrl+B' },
      { label: 'Toggle Panel', action: onToggleBottomPanel, shortcut: 'Ctrl+J' },
      { label: 'Split Editor Right', action: onToggleSplitEditor, shortcut: 'Ctrl+\\' },
      { label: 'Show Oxide Profiler & Benchmarks', action: onOpenRustProfiler, shortcut: 'Ctrl+Shift+B', divider: true },
    ],
    Go: [
      { label: 'Go to File...', action: onOpenQuickOpen, shortcut: 'Ctrl+P' },
      { label: 'Go to Definition (rust-analyzer)', action: () => {}, shortcut: 'F12' },
      { label: 'Go to Symbol in Workspace...', action: onOpenCommandPalette, shortcut: 'Ctrl+T' },
    ],
    Run: [
      { label: 'Start Debugging (Rust lldb)', action: () => onExecuteCommand('cargo run'), shortcut: 'F5' },
      { label: 'Run Without Debugging', action: () => onExecuteCommand('cargo run'), shortcut: 'Ctrl+F5' },
      { label: 'Run Rust Tests (cargo test)', action: () => onExecuteCommand('cargo test'), shortcut: 'Ctrl+T' },
    ],
    Terminal: [
      { label: 'New Terminal', action: onToggleBottomPanel, shortcut: 'Ctrl+`' },
      { label: 'Run Build Task (cargo build)', action: () => onExecuteCommand('cargo build --release') },
      { label: 'Run Linter (cargo clippy)', action: () => onExecuteCommand('cargo clippy') },
    ],
    'Rust Engine': [
      { label: 'Open Latency & Memory Profiler', action: onOpenRustProfiler },
      { label: 'Run Electron vs Rust Microbenchmark', action: () => onExecuteCommand('bench') },
      { label: 'Verify Tree-Sitter AST Cache', action: () => onExecuteCommand('cargo check') },
    ],
  };

  return (
    <header
      className="h-9 w-full flex items-center justify-between px-2 text-xs select-none border-b shrink-0 relative z-30"
      style={{
        backgroundColor: theme.titleBarBg,
        borderColor: theme.border,
        color: theme.textPrimary,
      }}
      onClick={(e) => {
        // If clicking outside menu, close active menu
        if ((e.target as HTMLElement).tagName !== 'BUTTON') {
          setActiveMenu(null);
        }
      }}
    >
      {/* Left: Window Controls + App Menu */}
      <div className="flex items-center gap-2">
        {settings.platformTitlebar === 'macos' ? (
          <div className="flex items-center gap-1.5 mr-2">
            <span className="w-3 h-3 rounded-full bg-[#ff5f56] border border-[#e0443e] inline-block shadow-sm hover:opacity-80 cursor-pointer" title="Close" />
            <span className="w-3 h-3 rounded-full bg-[#ffbd2e] border border-[#dea123] inline-block shadow-sm hover:opacity-80 cursor-pointer" title="Minimize" />
            <span className="w-3 h-3 rounded-full bg-[#27c93f] border border-[#1aab29] inline-block shadow-sm hover:opacity-80 cursor-pointer" title="Maximize" />
          </div>
        ) : (
          <div className="flex items-center gap-1 mr-2 text-orange-500 font-bold tracking-tight">
            <span>🦀</span>
          </div>
        )}

        {/* Brand Icon & Name */}
        <div className="flex items-center gap-1.5 font-medium text-xs mr-2 text-zinc-300">
          <span className="text-orange-500 font-black">OxideCode</span>
          <span className="text-[10px] text-zinc-500 px-1 py-0.2 bg-zinc-800/80 rounded border border-zinc-700/60">Rust v0.1.0</span>
        </div>

        {/* Menu Bar */}
        <nav className="flex items-center space-x-0.5">
          {Object.keys(menus).map((menuKey) => (
            <div key={menuKey} className="relative">
              <button
                type="button"
                onClick={() => setActiveMenu(activeMenu === menuKey ? null : menuKey)}
                onMouseEnter={() => activeMenu && setActiveMenu(menuKey)}
                className={`px-2 py-1 rounded transition-colors text-xs ${
                  activeMenu === menuKey
                    ? 'bg-zinc-700/60 text-white'
                    : 'text-zinc-300 hover:bg-zinc-800 hover:text-white'
                }`}
              >
                {menuKey}
              </button>

              {/* Dropdown Menu */}
              {activeMenu === menuKey && (
                <div
                  className="absolute top-full left-0 mt-1 w-64 rounded-md shadow-2xl py-1 text-xs border z-50 animate-in fade-in zoom-in-95 duration-100"
                  style={{
                    backgroundColor: theme.sidebarBg,
                    borderColor: theme.border,
                    color: theme.textPrimary,
                  }}
                >
                  {menus[menuKey].map((item, idx) => (
                    <React.Fragment key={idx}>
                      <button
                        type="button"
                        onClick={() => {
                          item.action();
                          setActiveMenu(null);
                        }}
                        className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-orange-600 hover:text-white transition-colors"
                      >
                        <span>{item.label}</span>
                        {item.shortcut && (
                          <span className="text-[10px] opacity-60 ml-2 font-mono">
                            {item.shortcut}
                          </span>
                        )}
                      </button>
                      {item.divider && (
                        <div
                          className="my-1 border-t"
                          style={{ borderColor: theme.border }}
                        />
                      )}
                    </React.Fragment>
                  ))}
                </div>
              )}
            </div>
          ))}
        </nav>
      </div>

      {/* Center: Quick Open / Search Palette Bar */}
      <div className="flex-1 max-w-lg mx-4">
        <button
          type="button"
          onClick={onOpenQuickOpen}
          className="w-full h-6 px-3 rounded flex items-center justify-between text-xs transition-colors border shadow-inner group"
          style={{
            backgroundColor: theme.editorBg,
            borderColor: theme.border,
            color: theme.textMuted,
          }}
          title="Search files, symbols, commands (Ctrl+P / Ctrl+Shift+P)"
        >
          <div className="flex items-center gap-2 truncate">
            <Search className="w-3.5 h-3.5 text-zinc-400 group-hover:text-orange-400" />
            <span className="truncate group-hover:text-zinc-200">
              oxide-editor — {activeFileName || 'src/main.rs'}
            </span>
          </div>
          <div className="flex items-center gap-1 font-mono text-[10px] text-zinc-500">
            <kbd className="px-1 py-0.5 bg-zinc-800 rounded border border-zinc-700">Ctrl</kbd>
            <kbd className="px-1 py-0.5 bg-zinc-800 rounded border border-zinc-700">P</kbd>
          </div>
        </button>
      </div>

      {/* Right: Rust Engine Telemetry Badge + Layout Controls */}
      <div className="flex items-center gap-2">
        {/* Rust Engine Telemetry Pill Button */}
        <button
          type="button"
          onClick={onOpenRustProfiler}
          className="flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono border border-orange-500/30 bg-orange-950/20 text-orange-300 hover:bg-orange-900/30 transition-colors"
          title="Click to view Rust Architecture & Live Latency Profiler"
        >
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-orange-500" />
          </span>
          <span className="font-semibold text-orange-400">0.8ms</span>
          <span className="text-zinc-400">·</span>
          <span className="text-zinc-300">22MB</span>
          <span className="text-orange-400/80 font-bold">wgpu</span>
        </button>

        {/* Layout Switchers */}
        <div className="flex items-center space-x-1 pl-1 border-l border-zinc-700/60">
          <button
            type="button"
            onClick={onToggleSidebar}
            className="p-1 rounded hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
            title="Toggle Primary Side Bar (Ctrl+B)"
            aria-label="Toggle Primary Side Bar"
          >
            <PanelLeft className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            onClick={onToggleBottomPanel}
            className="p-1 rounded hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
            title="Toggle Bottom Terminal Panel (Ctrl+J)"
            aria-label="Toggle Bottom Terminal Panel"
          >
            <PanelBottom className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            onClick={onToggleSplitEditor}
            className="p-1 rounded hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
            title="Split Editor Right (Ctrl+\)"
            aria-label="Split Editor Right"
          >
            <Columns className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </header>
  );
};
