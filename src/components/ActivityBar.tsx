import React, { useState } from 'react';
import {
  Files,
  Search,
  GitBranch,
  PlayCircle,
  Package,
  Activity,
  Image as ImageIcon,
  Settings as SettingsIcon,
  User,
  Sliders,
  Palette,
  Check,
} from 'lucide-react';
import { ThemeColors, THEMES } from '../styles/themes.ts';
import { EditorSettings, ThemeType } from '../types/editor.ts';

export type ActivityTab =
  | 'explorer'
  | 'search'
  | 'git'
  | 'debug'
  | 'extensions'
  | 'profiler'
  | 'mockups';

interface ActivityBarProps {
  theme: ThemeColors;
  activeTab: ActivityTab;
  onSelectTab: (tab: ActivityTab) => void;
  sidebarOpen: boolean;
  settings: EditorSettings;
  onUpdateSettings: (newSettings: Partial<EditorSettings>) => void;
  modifiedFilesCount: number;
}

export const ActivityBar: React.FC<ActivityBarProps> = ({
  theme,
  activeTab,
  onSelectTab,
  sidebarOpen,
  settings,
  onUpdateSettings,
  modifiedFilesCount,
}) => {
  const [showSettingsMenu, setShowSettingsMenu] = useState(false);
  const [showThemePicker, setShowThemePicker] = useState(false);

  const topItems: {
    id: ActivityTab;
    label: string;
    icon: React.ReactNode;
    badge?: number | string;
  }[] = [
    {
      id: 'explorer',
      label: 'Explorer (Ctrl+Shift+E)',
      icon: <Files className="w-5 h-5" />,
    },
    {
      id: 'search',
      label: 'Search (Ctrl+Shift+F)',
      icon: <Search className="w-5 h-5" />,
    },
    {
      id: 'git',
      label: 'Source Control: Git (Ctrl+Shift+G)',
      icon: <GitBranch className="w-5 h-5" />,
      badge: modifiedFilesCount > 0 ? modifiedFilesCount : undefined,
    },
    {
      id: 'debug',
      label: 'Run and Debug (Ctrl+Shift+D)',
      icon: <PlayCircle className="w-5 h-5" />,
    },
    {
      id: 'extensions',
      label: 'Extensions: Marketplace (Ctrl+Shift+X)',
      icon: <Package className="w-5 h-5" />,
      badge: 'Rust',
    },
    {
      id: 'profiler',
      label: 'Rust Performance Engine & Latency Profiler',
      icon: <Activity className="w-5 h-5 text-orange-400" />,
    },
    {
      id: 'mockups',
      label: 'Design Mockups & Architecture Visuals',
      icon: <ImageIcon className="w-5 h-5 text-amber-300" />,
      badge: '2',
    },
  ];

  return (
    <aside
      className="w-12 h-full flex flex-col justify-between items-center py-2 select-none border-r shrink-0 relative z-20"
      style={{
        backgroundColor: theme.activityBarBg,
        borderColor: theme.border,
      }}
      aria-label="Activity Bar"
    >
      {/* Top Icons */}
      <div className="flex flex-col items-center space-y-1 w-full">
        {topItems.map((item) => {
          const isActive = sidebarOpen && activeTab === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onSelectTab(item.id)}
              className={`relative w-full h-11 flex items-center justify-center transition-all group ${
                isActive
                  ? 'text-white'
                  : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40'
              }`}
              title={item.label}
              aria-label={item.label}
            >
              {/* Active Indicator bar */}
              {isActive && (
                <span
                  className="absolute left-0 top-1 bottom-1 w-0.5 rounded-r"
                  style={{ backgroundColor: theme.accent }}
                />
              )}

              {item.icon}

              {/* Badge */}
              {item.badge !== undefined && (
                <span
                  className="absolute top-2 right-1.5 px-1 py-0.2 min-w-4 text-[9px] font-bold rounded-full text-center shadow"
                  style={{
                    backgroundColor: theme.accent,
                    color: '#ffffff',
                  }}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Bottom Icons: Account & Settings */}
      <div className="flex flex-col items-center space-y-1 w-full relative">
        <button
          type="button"
          className="w-full h-10 flex items-center justify-center text-zinc-400 hover:text-white transition-colors"
          title="Rust Cloud Sync (Logged in as Oxide Developer)"
          aria-label="User Profile"
        >
          <User className="w-4 h-4" />
        </button>

        {/* Settings Button */}
        <div className="relative w-full">
          <button
            type="button"
            onClick={() => {
              setShowSettingsMenu(!showSettingsMenu);
              setShowThemePicker(false);
            }}
            className={`w-full h-10 flex items-center justify-center transition-colors ${
              showSettingsMenu ? 'text-white' : 'text-zinc-400 hover:text-white'
            }`}
            title="Manage Settings and Themes"
            aria-label="Manage Settings"
          >
            <SettingsIcon className="w-4 h-4" />
          </button>

          {/* Settings Popover */}
          {showSettingsMenu && (
            <div
              className="absolute bottom-2 left-14 w-60 rounded-md shadow-2xl py-1 text-xs border z-50 animate-in fade-in slide-in-from-left-2 duration-100"
              style={{
                backgroundColor: theme.sidebarBg,
                borderColor: theme.border,
                color: theme.textPrimary,
              }}
            >
              <div className="px-3 py-1 font-semibold text-zinc-400 border-b border-zinc-700/50 flex items-center justify-between text-[11px]">
                <span>OxideCode Settings</span>
                <span className="text-orange-400 font-mono">Rust Engine</span>
              </div>

              {/* Theme Selector Trigger */}
              <button
                type="button"
                onClick={() => setShowThemePicker(!showThemePicker)}
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <div className="flex items-center gap-2">
                  <Palette className="w-3.5 h-3.5 text-zinc-400" />
                  <span>Color Theme</span>
                </div>
                <span className="text-[10px] text-zinc-400 font-medium">
                  {THEMES[settings.theme].name}
                </span>
              </button>

              {/* Inlay Hints Toggle */}
              <button
                type="button"
                onClick={() =>
                  onUpdateSettings({ inlayHints: !settings.inlayHints })
                }
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <span>rust-analyzer Inlay Hints</span>
                <span
                  className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                    settings.inlayHints
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-zinc-800 text-zinc-400'
                  }`}
                >
                  {settings.inlayHints ? 'ON' : 'OFF'}
                </span>
              </button>

              {/* Format on Save */}
              <button
                type="button"
                onClick={() =>
                  onUpdateSettings({ formatOnSave: !settings.formatOnSave })
                }
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <span>rustfmt on Save</span>
                <span
                  className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                    settings.formatOnSave
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-zinc-800 text-zinc-400'
                  }`}
                >
                  {settings.formatOnSave ? 'ON' : 'OFF'}
                </span>
              </button>

              {/* GitLens Inline Blame */}
              <button
                type="button"
                onClick={() =>
                  onUpdateSettings({ gitLensBlame: !settings.gitLensBlame })
                }
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <span>GitLens Inline Blame</span>
                <span
                  className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                    settings.gitLensBlame
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-zinc-800 text-zinc-400'
                  }`}
                >
                  {settings.gitLensBlame ? 'ON' : 'OFF'}
                </span>
              </button>

              {/* GitHub Copilot Ghost Text */}
              <button
                type="button"
                onClick={() =>
                  onUpdateSettings({
                    copilotSuggestions: !settings.copilotSuggestions,
                  })
                }
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <span>Copilot Ghost Text</span>
                <span
                  className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                    settings.copilotSuggestions
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-zinc-800 text-zinc-400'
                  }`}
                >
                  {settings.copilotSuggestions ? 'ON' : 'OFF'}
                </span>
              </button>

              {/* Minimap Toggle */}
              <button
                type="button"
                onClick={() =>
                  onUpdateSettings({ minimap: !settings.minimap })
                }
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-zinc-700/50 transition-colors"
              >
                <span>Editor Minimap</span>
                <span
                  className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                    settings.minimap
                      ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                      : 'bg-zinc-800 text-zinc-400'
                  }`}
                >
                  {settings.minimap ? 'ON' : 'OFF'}
                </span>
              </button>

              {/* Sub-menu Theme Picker */}
              {showThemePicker && (
                <div
                  className="mt-1 pt-1 border-t border-zinc-700/50 bg-black/20 p-1 rounded"
                >
                  {(Object.keys(THEMES) as ThemeType[]).map((themeKey) => (
                    <button
                      key={themeKey}
                      type="button"
                      onClick={() => {
                        onUpdateSettings({ theme: themeKey });
                        setShowThemePicker(false);
                      }}
                      className="w-full text-left px-2 py-1 flex items-center justify-between rounded hover:bg-orange-600 hover:text-white transition-colors text-xs"
                    >
                      <span>{THEMES[themeKey].name}</span>
                      {settings.theme === themeKey && (
                        <Check className="w-3.5 h-3.5 text-orange-400" />
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </aside>
  );
};
