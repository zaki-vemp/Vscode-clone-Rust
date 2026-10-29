export interface WorkspaceFile {
  id: string;
  name: string;
  path: string;
  language: 'rust' | 'toml' | 'markdown' | 'typescript' | 'json';
  content: string;
  isModified?: boolean;
  isNew?: boolean;
}

export interface WorkspaceFolder {
  name: string;
  files: WorkspaceFile[];
}

export interface OpenTab {
  fileId: string;
  title: string;
  path: string;
  isDirty?: boolean;
  isPinned?: boolean;
}

export interface Diagnostic {
  id: string;
  fileId: string;
  filePath: string;
  line: number;
  column: number;
  message: string;
  severity: 'error' | 'warning' | 'info';
  source: 'rust-analyzer' | 'rustc' | 'clippy';
  codeRule?: string;
  quickFix?: {
    title: string;
    action: string;
  };
}

export interface ExtensionItem {
  id: string;
  name: string;
  displayName: string;
  publisher: string;
  version: string;
  description: string;
  downloads: string;
  rating: number;
  iconBg: string;
  iconSymbol: string;
  installed: boolean;
  enabled: boolean;
  category: 'Languages' | 'Linters' | 'Git' | 'AI' | 'Formatters' | 'Themes';
  features: string[];
  wasmRuntime: boolean;
  settings?: Record<string, boolean | string | number>;
}

export type ThemeType = 'dark-modern' | 'tokyo-night' | 'catppuccin' | 'monokai-pro' | 'rust-oxide';

export interface EditorSettings {
  theme: ThemeType;
  inlayHints: boolean;
  formatOnSave: boolean;
  minimap: boolean;
  gitLensBlame: boolean;
  copilotSuggestions: boolean;
  fontSize: number;
  platformTitlebar: 'macos' | 'windows' | 'linux';
  rustTelemetryHud: boolean;
  lineNumbers: boolean;
  tabSize: number;
}
