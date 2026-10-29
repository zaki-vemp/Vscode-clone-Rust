import { ThemeType } from '../types/editor.ts';

export interface ThemeColors {
  name: string;
  windowBg: string;
  titleBarBg: string;
  activityBarBg: string;
  sidebarBg: string;
  editorBg: string;
  statusBarBg: string;
  statusBarFg: string;
  tabActiveBg: string;
  tabInactiveBg: string;
  panelBg: string;
  border: string;
  accent: string;
  accentHover: string;
  textPrimary: string;
  textMuted: string;
  editorLineNumber: string;
  editorSelection: string;
}

export const THEMES: Record<ThemeType, ThemeColors> = {
  'dark-modern': {
    name: 'VS Code Dark Modern',
    windowBg: '#181818',
    titleBarBg: '#1f1f1f',
    activityBarBg: '#181818',
    sidebarBg: '#181818',
    editorBg: '#1f1f1f',
    statusBarBg: '#007acc',
    statusBarFg: '#ffffff',
    tabActiveBg: '#1f1f1f',
    tabInactiveBg: '#181818',
    panelBg: '#181818',
    border: '#2b2b2b',
    accent: '#007acc',
    accentHover: '#0062a3',
    textPrimary: '#cccccc',
    textMuted: '#858585',
    editorLineNumber: '#6e7681',
    editorSelection: '#264f78',
  },
  'tokyo-night': {
    name: 'Tokyo Night',
    windowBg: '#1a1b26',
    titleBarBg: '#16161e',
    activityBarBg: '#16161e',
    sidebarBg: '#1a1b26',
    editorBg: '#1a1b26',
    statusBarBg: '#7aa2f7',
    statusBarFg: '#15161e',
    tabActiveBg: '#1a1b26',
    tabInactiveBg: '#16161e',
    panelBg: '#16161e',
    border: '#292e42',
    accent: '#7aa2f7',
    accentHover: '#bb9af7',
    textPrimary: '#c0caf5',
    textMuted: '#565f89',
    editorLineNumber: '#3b4261',
    editorSelection: '#283457',
  },
  'catppuccin': {
    name: 'Catppuccin Mocha',
    windowBg: '#1e1e2e',
    titleBarBg: '#181825',
    activityBarBg: '#11111b',
    sidebarBg: '#181825',
    editorBg: '#1e1e2e',
    statusBarBg: '#cba6f7',
    statusBarFg: '#11111b',
    tabActiveBg: '#1e1e2e',
    tabInactiveBg: '#181825',
    panelBg: '#181825',
    border: '#313244',
    accent: '#cba6f7',
    accentHover: '#b4befe',
    textPrimary: '#cdd6f4',
    textMuted: '#6c7086',
    editorLineNumber: '#45475a',
    editorSelection: '#45475a',
  },
  'monokai-pro': {
    name: 'Monokai Pro',
    windowBg: '#222222',
    titleBarBg: '#1d1d1d',
    activityBarBg: '#191919',
    sidebarBg: '#222222',
    editorBg: '#272822',
    statusBarBg: '#ffd866',
    statusBarFg: '#222222',
    tabActiveBg: '#272822',
    tabInactiveBg: '#1d1d1d',
    panelBg: '#1d1d1d',
    border: '#383830',
    accent: '#ffd866',
    accentHover: '#fc9867',
    textPrimary: '#f8f8f2',
    textMuted: '#75715e',
    editorLineNumber: '#49483e',
    editorSelection: '#49483e',
  },
  'rust-oxide': {
    name: 'Rust Oxide Dark',
    windowBg: '#15141b',
    titleBarBg: '#111015',
    activityBarBg: '#0e0d12',
    sidebarBg: '#15141b',
    editorBg: '#17161f',
    statusBarBg: '#ea580c',
    statusBarFg: '#ffffff',
    tabActiveBg: '#17161f',
    tabInactiveBg: '#111015',
    panelBg: '#111015',
    border: '#2a2736',
    accent: '#ea580c',
    accentHover: '#c2410c',
    textPrimary: '#e4e4e7',
    textMuted: '#71717a',
    editorLineNumber: '#3f3f46',
    editorSelection: '#3b251a',
  },
};
