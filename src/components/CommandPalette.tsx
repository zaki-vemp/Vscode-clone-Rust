import React, { useState, useEffect, useRef } from 'react';
import {
  Search,
  Command,
  FileCode,
  Palette,
  Play,
  Zap,
  Sliders,
  Check,
  ChevronRight,
  Layers,
} from 'lucide-react';
import { ThemeColors, THEMES } from '../styles/themes.ts';
import { EditorSettings, ThemeType } from '../types/editor.ts';

interface CommandItem {
  id: string;
  category: string;
  title: string;
  shortcut?: string;
  icon?: React.ReactNode;
  action: () => void;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  theme: ThemeColors;
  settings: EditorSettings;
  onUpdateSettings: (s: Partial<EditorSettings>) => void;
  onRunTerminalCommand: (cmd: string) => void;
  onOpenRustProfiler: () => void;
  onOpenMockupModal: (imageSrc: string, title: string) => void;
  onToggleSplit: () => void;
  onSelectFile: (fileId: string) => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  theme,
  settings,
  onUpdateSettings,
  onRunTerminalCommand,
  onOpenRustProfiler,
  onOpenMockupModal,
  onToggleSplit,
  onSelectFile,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const allCommands: CommandItem[] = [
    {
      id: 'cmd-format',
      category: 'Rust',
      title: 'Format Document (rustfmt Tree-Sitter AST)',
      shortcut: 'Shift+Alt+F',
      icon: <Zap className="w-3.5 h-3.5 text-orange-400" />,
      action: () => onRunTerminalCommand('cargo check'),
    },
    {
      id: 'cmd-check',
      category: 'Cargo',
      title: 'Cargo: Check (rust-analyzer)',
      shortcut: 'Ctrl+Shift+B',
      icon: <Play className="w-3.5 h-3.5 text-emerald-400" />,
      action: () => onRunTerminalCommand('cargo check'),
    },
    {
      id: 'cmd-test',
      category: 'Cargo',
      title: 'Cargo: Test Suite',
      shortcut: 'Ctrl+T',
      icon: <Play className="w-3.5 h-3.5 text-emerald-400" />,
      action: () => onRunTerminalCommand('cargo test'),
    },
    {
      id: 'cmd-clippy',
      category: 'Cargo',
      title: 'Cargo: Clippy Linter & Borrow Checker',
      icon: <Zap className="w-3.5 h-3.5 text-amber-400" />,
      action: () => onRunTerminalCommand('cargo clippy'),
    },
    {
      id: 'cmd-bench',
      category: 'Oxide Profiler',
      title: 'Run Latency Microbenchmark (Rust vs Electron)',
      icon: <Zap className="w-3.5 h-3.5 text-orange-400" />,
      action: () => onRunTerminalCommand('bench'),
    },
    {
      id: 'cmd-profiler',
      category: 'Oxide Profiler',
      title: 'Open Rust Architecture & Profiler Modal',
      icon: <Layers className="w-3.5 h-3.5 text-purple-400" />,
      action: onOpenRustProfiler,
    },
    {
      id: 'cmd-mockups',
      category: 'Design',
      title: 'Show Generated Desktop UI Mockups (macOS & Profiler)',
      icon: <Palette className="w-3.5 h-3.5 text-amber-300" />,
      action: () =>
        onOpenMockupModal(
          '/src/assets/images/rust_vscode_desktop_mockup_1793221900017.jpg',
          'OxideCode Desktop Environment (macOS Native Shell)'
        ),
    },
    {
      id: 'cmd-inlay-hints',
      category: 'View',
      title: `Toggle Inlay Hints (Currently: ${settings.inlayHints ? 'ON' : 'OFF'})`,
      icon: <Sliders className="w-3.5 h-3.5 text-blue-400" />,
      action: () => onUpdateSettings({ inlayHints: !settings.inlayHints }),
    },
    {
      id: 'cmd-split',
      category: 'View',
      title: 'Split Editor (Side-by-Side)',
      shortcut: 'Ctrl+\\',
      icon: <ChevronRight className="w-3.5 h-3.5" />,
      action: onToggleSplit,
    },
    {
      id: 'cmd-minimap',
      category: 'View',
      title: `Toggle Minimap (Currently: ${settings.minimap ? 'ON' : 'OFF'})`,
      icon: <Sliders className="w-3.5 h-3.5 text-blue-400" />,
      action: () => onUpdateSettings({ minimap: !settings.minimap }),
    },
    {
      id: 'cmd-theme-tokyo',
      category: 'Theme',
      title: 'Preferences: Color Theme -> Tokyo Night',
      icon: <Palette className="w-3.5 h-3.5 text-purple-400" />,
      action: () => onUpdateSettings({ theme: 'tokyo-night' }),
    },
    {
      id: 'cmd-theme-modern',
      category: 'Theme',
      title: 'Preferences: Color Theme -> VS Code Dark Modern',
      icon: <Palette className="w-3.5 h-3.5 text-blue-400" />,
      action: () => onUpdateSettings({ theme: 'dark-modern' }),
    },
    {
      id: 'cmd-theme-oxide',
      category: 'Theme',
      title: 'Preferences: Color Theme -> Rust Oxide Dark',
      icon: <Palette className="w-3.5 h-3.5 text-orange-400" />,
      action: () => onUpdateSettings({ theme: 'rust-oxide' }),
    },
    {
      id: 'cmd-theme-catppuccin',
      category: 'Theme',
      title: 'Preferences: Color Theme -> Catppuccin Mocha',
      icon: <Palette className="w-3.5 h-3.5 text-pink-400" />,
      action: () => onUpdateSettings({ theme: 'catppuccin' }),
    },
    {
      id: 'cmd-open-main',
      category: 'File',
      title: 'Go to File: src/main.rs',
      icon: <FileCode className="w-3.5 h-3.5 text-orange-400" />,
      action: () => onSelectFile('f-main-rs'),
    },
    {
      id: 'cmd-open-cargo',
      category: 'File',
      title: 'Go to File: Cargo.toml',
      icon: <FileCode className="w-3.5 h-3.5 text-amber-400" />,
      action: () => onSelectFile('f-cargo-toml'),
    },
    {
      id: 'cmd-open-readme',
      category: 'File',
      title: 'Go to File: README.md',
      icon: <FileCode className="w-3.5 h-3.5 text-blue-400" />,
      action: () => onSelectFile('f-readme'),
    },
  ];

  const filtered = allCommands.filter((c) =>
    `${c.category} ${c.title}`.toLowerCase().includes(query.toLowerCase())
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % filtered.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + filtered.length) % filtered.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (filtered[selectedIndex]) {
        filtered[selectedIndex].action();
        onClose();
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/60 z-50 flex items-start justify-center pt-12 animate-in fade-in duration-100"
      onClick={onClose}
    >
      <div
        className="w-[560px] rounded-lg shadow-2xl border overflow-hidden flex flex-col max-h-[460px] animate-in zoom-in-95 duration-100 text-xs"
        style={{
          backgroundColor: theme.sidebarBg,
          borderColor: theme.border,
          color: theme.textPrimary,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="p-2 border-b flex items-center gap-2" style={{ borderColor: theme.border }}>
          <Search className="w-4 h-4 text-orange-400 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or file name (e.g., 'cargo', 'format', 'theme')..."
            className="w-full bg-transparent text-sm text-white outline-none placeholder-zinc-500 font-sans"
          />
        </div>

        {/* Command list */}
        <div className="overflow-y-auto p-1 space-y-0.5">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-zinc-500 text-xs">No matching commands found</div>
          ) : (
            filtered.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={cmd.id}
                  onClick={() => {
                    cmd.action();
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`px-3 py-2 rounded flex items-center justify-between cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-orange-600 text-white'
                      : 'text-zinc-300 hover:bg-zinc-800/60'
                  }`}
                >
                  <div className="flex items-center gap-2.5 truncate">
                    {cmd.icon}
                    <div className="flex items-center gap-1.5 truncate">
                      <span className="font-semibold text-zinc-400 text-[11px]">
                        {cmd.category}:
                      </span>
                      <span className="truncate font-medium">{cmd.title}</span>
                    </div>
                  </div>

                  {cmd.shortcut && (
                    <span
                      className={`text-[10px] font-mono px-1.5 py-0.5 rounded ml-2 shrink-0 ${
                        isSelected
                          ? 'bg-orange-700 text-white'
                          : 'bg-zinc-800 text-zinc-400 border border-zinc-700'
                      }`}
                    >
                      {cmd.shortcut}
                    </span>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
