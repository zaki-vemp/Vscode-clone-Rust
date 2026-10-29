import React, { useState } from 'react';
import {
  FolderPlus,
  FilePlus,
  RefreshCw,
  MoreHorizontal,
  ChevronDown,
  ChevronRight,
  FileCode,
  FileText,
  File,
  Check,
  Search,
  ArrowRight,
  Play,
  RotateCcw,
  Plus,
  Trash2,
  GitCommit,
  CheckCircle2,
  AlertTriangle,
  Zap,
  Cpu,
  Layers,
  Sparkles,
  ExternalLink,
  Sliders,
} from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';
import { WorkspaceFile, ExtensionItem, EditorSettings } from '../types/editor.ts';
import { ActivityTab } from './ActivityBar.tsx';

interface SideBarProps {
  theme: ThemeColors;
  activeTab: ActivityTab;
  files: WorkspaceFile[];
  activeFileId: string;
  onSelectFile: (fileId: string) => void;
  onCreateFile: (name: string) => void;
  onDeleteFile: (fileId: string) => void;
  extensions: ExtensionItem[];
  onToggleExtension: (id: string) => void;
  onToggleInstallExtension: (id: string) => void;
  settings: EditorSettings;
  onUpdateSettings: (s: Partial<EditorSettings>) => void;
  onRunTerminalCommand: (cmd: string) => void;
  onOpenMockupModal: (imageSrc: string, title: string) => void;
}

export const SideBar: React.FC<SideBarProps> = ({
  theme,
  activeTab,
  files,
  activeFileId,
  onSelectFile,
  onCreateFile,
  onDeleteFile,
  extensions,
  onToggleExtension,
  onToggleInstallExtension,
  settings,
  onUpdateSettings,
  onRunTerminalCommand,
  onOpenMockupModal,
}) => {
  // Explorer state
  const [srcFolderOpen, setSrcFolderOpen] = useState(true);
  const [bufferFolderOpen, setBufferFolderOpen] = useState(true);
  const [newFileInputOpen, setNewFileInputOpen] = useState(false);
  const [newFileName, setNewFileName] = useState('');

  // Search state
  const [searchQuery, setSearchQuery] = useState('');
  const [replaceQuery, setReplaceQuery] = useState('');
  const [matchCase, setMatchCase] = useState(false);
  const [showReplace, setShowReplace] = useState(false);

  // Git state
  const [commitMessage, setCommitMessage] = useState('');
  const [stagedFiles, setStagedFiles] = useState<string[]>(['f-cargo-toml']);

  // Extensions filter
  const [extSearch, setExtSearch] = useState('');
  const [extTab, setExtTab] = useState<'all' | 'installed' | 'recommended'>('all');

  const getFileIcon = (fileName: string) => {
    if (fileName.endsWith('.rs')) {
      return <span className="text-orange-500 font-bold text-xs select-none">🦀</span>;
    }
    if (fileName.endsWith('.toml')) {
      return <span className="text-amber-400 font-bold text-xs select-none">⚙️</span>;
    }
    if (fileName.endsWith('.md')) {
      return <span className="text-blue-400 font-bold text-xs select-none">📄</span>;
    }
    return <FileText className="w-4 h-4 text-zinc-400" />;
  };

  const handleCreateNewFile = (e: React.FormEvent) => {
    e.preventDefault();
    if (newFileName.trim()) {
      onCreateFile(newFileName.trim());
      setNewFileName('');
      setNewFileInputOpen(false);
    }
  };

  const filteredExtensions = extensions.filter((ext) => {
    const matchesSearch =
      ext.displayName.toLowerCase().includes(extSearch.toLowerCase()) ||
      ext.description.toLowerCase().includes(extSearch.toLowerCase()) ||
      ext.category.toLowerCase().includes(extSearch.toLowerCase());

    if (extTab === 'installed') return matchesSearch && ext.installed;
    if (extTab === 'recommended') return matchesSearch && !ext.installed;
    return matchesSearch;
  });

  // Calculate search matches across files
  const searchMatches = searchQuery.trim()
    ? files
        .map((f) => {
          const lines = f.content.split('\n');
          const matchedLines = lines
            .map((line, idx) => ({ line, num: idx + 1 }))
            .filter((l) =>
              matchCase
                ? l.line.includes(searchQuery)
                : l.line.toLowerCase().includes(searchQuery.toLowerCase())
            );
          return { file: f, matches: matchedLines };
        })
        .filter((r) => r.matches.length > 0)
    : [];

  return (
    <aside
      className="w-64 h-full flex flex-col border-r select-none overflow-hidden shrink-0 text-xs"
      style={{
        backgroundColor: theme.sidebarBg,
        borderColor: theme.border,
        color: theme.textPrimary,
      }}
      aria-label="Side Bar"
    >
      {/* Side Bar Header */}
      <div className="h-9 px-4 flex items-center justify-between border-b shrink-0" style={{ borderColor: theme.border }}>
        <span className="font-semibold uppercase tracking-wider text-[11px] text-zinc-400">
          {activeTab === 'explorer' && 'Explorer'}
          {activeTab === 'search' && 'Search'}
          {activeTab === 'git' && 'Source Control'}
          {activeTab === 'debug' && 'Run and Debug'}
          {activeTab === 'extensions' && 'Extensions'}
          {activeTab === 'profiler' && 'Rust Engine Profiler'}
          {activeTab === 'mockups' && 'Design Mockups'}
        </span>

        {/* Header Action Buttons */}
        <div className="flex items-center space-x-1 text-zinc-400">
          {activeTab === 'explorer' && (
            <>
              <button
                type="button"
                onClick={() => setNewFileInputOpen(true)}
                className="p-1 hover:text-white rounded hover:bg-zinc-700/40"
                title="New File"
              >
                <FilePlus className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => onRunTerminalCommand('cargo check')}
                className="p-1 hover:text-white rounded hover:bg-zinc-700/40"
                title="Refresh Workspace (cargo check)"
              >
                <RefreshCw className="w-3.5 h-3.5" />
              </button>
            </>
          )}
          {activeTab === 'git' && (
            <button
              type="button"
              onClick={() => onRunTerminalCommand('git status')}
              className="p-1 hover:text-white rounded hover:bg-zinc-700/40"
              title="Refresh Git Status"
            >
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
          )}
          {activeTab === 'profiler' && (
            <button
              type="button"
              onClick={() => onRunTerminalCommand('bench')}
              className="p-1 text-orange-400 hover:text-orange-300 rounded hover:bg-zinc-700/40"
              title="Run Microbenchmark in Terminal"
            >
              <Zap className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Side Bar Content Body */}
      <div className="flex-1 overflow-y-auto">
        {/* ================= EXPLORER VIEW ================= */}
        {activeTab === 'explorer' && (
          <div className="py-2">
            {/* Section Header: OXIDE-EDITOR */}
            <div className="px-3 py-1 flex items-center justify-between text-[11px] font-bold text-zinc-400 uppercase tracking-wider">
              <div className="flex items-center gap-1">
                <ChevronDown className="w-3.5 h-3.5" />
                <span>OXIDE-EDITOR (RUST WORKSPACE)</span>
              </div>
            </div>

            {/* New File Inline Prompt */}
            {newFileInputOpen && (
              <form onSubmit={handleCreateNewFile} className="px-4 py-1">
                <div className="flex items-center gap-1 bg-zinc-900 border border-orange-500 rounded px-2 py-0.5">
                  <span className="text-[10px] text-orange-400">🦀</span>
                  <input
                    type="text"
                    value={newFileName}
                    onChange={(e) => setNewFileName(e.target.value)}
                    placeholder="filename.rs"
                    autoFocus
                    className="w-full bg-transparent text-xs text-white outline-none"
                    onBlur={() => setNewFileInputOpen(false)}
                  />
                </div>
              </form>
            )}

            {/* Tree Items */}
            <div className="mt-1">
              {/* Folder: src */}
              <div>
                <button
                  type="button"
                  onClick={() => setSrcFolderOpen(!srcFolderOpen)}
                  className="w-full px-4 py-1 flex items-center gap-1.5 text-zinc-300 hover:bg-zinc-800/50 transition-colors"
                >
                  {srcFolderOpen ? (
                    <ChevronDown className="w-3.5 h-3.5 text-zinc-400" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5 text-zinc-400" />
                  )}
                  <span className="font-semibold text-zinc-300">src</span>
                  <span className="text-[10px] text-zinc-500 ml-auto">Rust source</span>
                </button>

                {srcFolderOpen && (
                  <div className="pl-6 space-y-0.5">
                    {files
                      .filter((f) => f.path.startsWith('src/'))
                      .map((file) => {
                        const isSelected = file.id === activeFileId;
                        return (
                          <div
                            key={file.id}
                            onClick={() => onSelectFile(file.id)}
                            className={`px-3 py-1 flex items-center justify-between rounded cursor-pointer group transition-colors ${
                              isSelected
                                ? 'bg-orange-600/20 text-orange-200 border-l-2 border-orange-500'
                                : 'text-zinc-300 hover:bg-zinc-800/40 hover:text-white'
                            }`}
                          >
                            <div className="flex items-center gap-2 truncate">
                              {getFileIcon(file.name)}
                              <span className="truncate">{file.name}</span>
                            </div>

                            <div className="flex items-center gap-1">
                              {file.isModified && (
                                <span className="text-[10px] font-bold text-amber-400" title="Modified in working tree">
                                  M
                                </span>
                              )}
                              {file.isNew && (
                                <span className="text-[10px] font-bold text-emerald-400" title="Untracked new file">
                                  U
                                </span>
                              )}
                              <button
                                type="button"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  onDeleteFile(file.id);
                                }}
                                className="opacity-0 group-hover:opacity-100 hover:text-red-400 p-0.5"
                                title="Delete file"
                              >
                                <Trash2 className="w-3 h-3" />
                              </button>
                            </div>
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>

              {/* Root Level Files */}
              <div className="mt-1 space-y-0.5">
                {files
                  .filter((f) => !f.path.startsWith('src/'))
                  .map((file) => {
                    const isSelected = file.id === activeFileId;
                    return (
                      <div
                        key={file.id}
                        onClick={() => onSelectFile(file.id)}
                        className={`px-4 py-1 flex items-center justify-between cursor-pointer group transition-colors ${
                          isSelected
                            ? 'bg-orange-600/20 text-orange-200 border-l-2 border-orange-500'
                            : 'text-zinc-300 hover:bg-zinc-800/40 hover:text-white'
                        }`}
                      >
                        <div className="flex items-center gap-2 truncate">
                          {getFileIcon(file.name)}
                          <span className="truncate">{file.name}</span>
                        </div>
                        {file.isModified && (
                          <span className="text-[10px] font-bold text-amber-400">M</span>
                        )}
                      </div>
                    );
                  })}
              </div>
            </div>

            {/* Outline section */}
            <div className="mt-6 border-t pt-2" style={{ borderColor: theme.border }}>
              <div className="px-3 py-1 flex items-center justify-between text-[11px] font-bold text-zinc-400 uppercase">
                <div className="flex items-center gap-1">
                  <ChevronRight className="w-3.5 h-3.5" />
                  <span>OUTLINE (rust-analyzer)</span>
                </div>
              </div>
              <div className="px-6 py-1 text-[11px] text-zinc-500">
                <div>struct EditorState</div>
                <div>fn main() -&gt; Result&lt;()&gt;</div>
                <div>struct RopeBuffer</div>
              </div>
            </div>
          </div>
        )}

        {/* ================= SEARCH VIEW ================= */}
        {activeTab === 'search' && (
          <div className="p-3 space-y-3">
            <div className="space-y-1.5">
              <div className="relative">
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search code across files..."
                  className="w-full bg-zinc-900 border border-zinc-700 rounded px-2.5 py-1.5 text-xs text-white placeholder-zinc-500 focus:border-orange-500 outline-none"
                />
              </div>

              {showReplace && (
                <div className="relative">
                  <input
                    type="text"
                    value={replaceQuery}
                    onChange={(e) => setReplaceQuery(e.target.value)}
                    placeholder="Replace with..."
                    className="w-full bg-zinc-900 border border-zinc-700 rounded px-2.5 py-1.5 text-xs text-white placeholder-zinc-500 focus:border-orange-500 outline-none"
                  />
                </div>
              )}

              <div className="flex items-center justify-between text-zinc-400 text-[11px]">
                <button
                  type="button"
                  onClick={() => setShowReplace(!showReplace)}
                  className="hover:text-white"
                >
                  {showReplace ? 'Hide Replace' : 'Toggle Replace'}
                </button>
                <button
                  type="button"
                  onClick={() => setMatchCase(!matchCase)}
                  className={`px-1.5 py-0.5 rounded border ${
                    matchCase ? 'bg-orange-600 text-white border-orange-500' : 'bg-zinc-800 border-zinc-700'
                  }`}
                  title="Match Case"
                >
                  Aa
                </button>
              </div>
            </div>

            {/* Search Results */}
            <div className="space-y-2">
              <div className="text-[11px] text-zinc-400 font-medium">
                {searchQuery
                  ? `${searchMatches.reduce((acc, curr) => acc + curr.matches.length, 0)} results found`
                  : 'Type a query to search'}
              </div>

              {searchMatches.map((res) => (
                <div key={res.file.id} className="space-y-1">
                  <div
                    onClick={() => onSelectFile(res.file.id)}
                    className="flex items-center gap-1.5 font-semibold text-zinc-200 cursor-pointer hover:text-orange-400"
                  >
                    {getFileIcon(res.file.name)}
                    <span>{res.file.path}</span>
                    <span className="text-[10px] text-zinc-500 ml-auto">
                      {res.matches.length}
                    </span>
                  </div>
                  <div className="pl-4 space-y-0.5">
                    {res.matches.map((m, idx) => (
                      <div
                        key={idx}
                        onClick={() => onSelectFile(res.file.id)}
                        className="py-0.5 px-1.5 rounded hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 cursor-pointer font-mono text-[11px] truncate"
                      >
                        <span className="text-orange-400 mr-2">{m.num}:</span>
                        <span>{m.line.trim()}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ================= SOURCE CONTROL (GIT) ================= */}
        {activeTab === 'git' && (
          <div className="p-3 space-y-3">
            {/* Commit Message Box */}
            <div className="space-y-2">
              <textarea
                value={commitMessage}
                onChange={(e) => setCommitMessage(e.target.value)}
                placeholder="Commit message (Ctrl+Enter to commit)..."
                rows={3}
                className="w-full bg-zinc-900 border border-zinc-700 rounded p-2 text-xs text-white placeholder-zinc-500 focus:border-orange-500 outline-none resize-none font-mono"
              />
              <button
                type="button"
                onClick={() => {
                  if (commitMessage.trim()) {
                    onRunTerminalCommand(`git commit -m "${commitMessage.trim()}"`);
                    setCommitMessage('');
                  } else {
                    onRunTerminalCommand('git status');
                  }
                }}
                className="w-full py-1.5 bg-orange-600 hover:bg-orange-500 text-white rounded font-medium flex items-center justify-center gap-1.5 transition-colors"
              >
                <GitCommit className="w-3.5 h-3.5" />
                <span>Commit &amp; Push</span>
              </button>
            </div>

            {/* Staged Changes */}
            <div className="space-y-1">
              <div className="flex items-center justify-between text-[11px] font-bold text-zinc-400 uppercase tracking-wider">
                <span>STAGED CHANGES ({stagedFiles.length})</span>
                <span className="text-zinc-500">git add</span>
              </div>
              {stagedFiles.map((fid) => {
                const file = files.find((f) => f.id === fid) || files[0];
                return (
                  <div
                    key={fid}
                    onClick={() => onSelectFile(file.id)}
                    className="flex items-center justify-between px-2 py-1 rounded hover:bg-zinc-800/60 cursor-pointer text-zinc-300"
                  >
                    <div className="flex items-center gap-1.5">
                      {getFileIcon(file.name)}
                      <span>{file.name}</span>
                    </div>
                    <span className="text-emerald-400 font-bold text-[10px]">A</span>
                  </div>
                );
              })}
            </div>

            {/* Working Tree Changes */}
            <div className="space-y-1">
              <div className="flex items-center justify-between text-[11px] font-bold text-zinc-400 uppercase tracking-wider">
                <span>CHANGES (2)</span>
                <button
                  type="button"
                  onClick={() => onRunTerminalCommand('git status')}
                  className="text-orange-400 hover:underline"
                >
                  Stage All
                </button>
              </div>
              <div
                onClick={() => onSelectFile('f-main-rs')}
                className="flex items-center justify-between px-2 py-1 rounded hover:bg-zinc-800/60 cursor-pointer text-zinc-300"
              >
                <div className="flex items-center gap-1.5">
                  <span className="text-orange-500 font-bold text-xs">🦀</span>
                  <span>src/main.rs</span>
                </div>
                <span className="text-amber-400 font-bold text-[10px]">M</span>
              </div>
              <div
                onClick={() => onSelectFile('f-cargo-toml')}
                className="flex items-center justify-between px-2 py-1 rounded hover:bg-zinc-800/60 cursor-pointer text-zinc-300"
              >
                <div className="flex items-center gap-1.5">
                  <span className="text-amber-400 font-bold text-xs">⚙️</span>
                  <span>Cargo.toml</span>
                </div>
                <span className="text-amber-400 font-bold text-[10px]">M</span>
              </div>
            </div>

            {/* GitLens Extension Integration Notice */}
            <div className="mt-4 p-2.5 rounded bg-zinc-900/80 border border-sky-900/40 text-[11px] space-y-1">
              <div className="flex items-center gap-1.5 text-sky-400 font-semibold">
                <span>🌱</span>
                <span>GitLens Rust Extension Active</span>
              </div>
              <p className="text-zinc-400 text-[10px] leading-relaxed">
                Zero-copy libgit2 bindings provide instant commit blame and line authorship directly in the editor.
              </p>
            </div>
          </div>
        )}

        {/* ================= RUN & DEBUG ================= */}
        {activeTab === 'debug' && (
          <div className="p-3 space-y-3">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => onRunTerminalCommand('cargo run')}
                  className="flex-1 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded font-medium flex items-center justify-center gap-1.5 transition-colors"
                >
                  <Play className="w-3.5 h-3.5 fill-current" />
                  <span>Launch Oxide Native (F5)</span>
                </button>
                <button
                  type="button"
                  onClick={() => onRunTerminalCommand('cargo test')}
                  className="p-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded border border-zinc-700"
                  title="Run Rust tests"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                </button>
              </div>

              <div className="text-[11px] text-zinc-400 border border-zinc-800 rounded p-2 bg-zinc-900/60 space-y-1 font-mono">
                <div className="text-zinc-300 font-bold">Debug Configuration:</div>
                <div className="text-orange-400">target/debug/oxide-code</div>
                <div className="text-zinc-500 text-[10px]">Debugger: LLDB Rust Native v18</div>
              </div>
            </div>

            <div className="space-y-1 border-t pt-2" style={{ borderColor: theme.border }}>
              <div className="text-[11px] font-bold text-zinc-400 uppercase">CALL STACK</div>
              <div className="text-[11px] text-zinc-500 font-mono">oxide_core::main() at src/main.rs:26</div>
              <div className="text-[11px] text-zinc-500 font-mono">tokio::runtime::park()</div>
            </div>

            <div className="space-y-1 border-t pt-2" style={{ borderColor: theme.border }}>
              <div className="text-[11px] font-bold text-zinc-400 uppercase">BREAKPOINTS</div>
              <div className="flex items-center gap-1.5 text-zinc-300 text-[11px]">
                <span className="w-2 h-2 rounded-full bg-red-500 inline-block" />
                <span>main.rs:26 (main loop entry)</span>
              </div>
            </div>
          </div>
        )}

        {/* ================= EXTENSIONS MARKETPLACE ================= */}
        {activeTab === 'extensions' && (
          <div className="p-3 space-y-3">
            {/* Search Input */}
            <div className="relative">
              <input
                type="text"
                value={extSearch}
                onChange={(e) => setExtSearch(e.target.value)}
                placeholder="Search VS Code Extensions in Marketplace..."
                className="w-full bg-zinc-900 border border-zinc-700 rounded px-2.5 py-1.5 text-xs text-white placeholder-zinc-500 focus:border-orange-500 outline-none"
              />
            </div>

            {/* Filter Tabs */}
            <div className="flex items-center gap-1 p-0.5 bg-zinc-900 rounded border border-zinc-800">
              {(['all', 'installed', 'recommended'] as const).map((tab) => (
                <button
                  key={tab}
                  type="button"
                  onClick={() => setExtTab(tab)}
                  className={`flex-1 py-1 text-[10px] font-semibold capitalize rounded transition-colors ${
                    extTab === tab
                      ? 'bg-zinc-800 text-white shadow'
                      : 'text-zinc-400 hover:text-zinc-200'
                  }`}
                >
                  {tab}
                </button>
              ))}
            </div>

            {/* Extensions List */}
            <div className="space-y-2">
              {filteredExtensions.map((ext) => (
                <div
                  key={ext.id}
                  className="p-2.5 rounded border border-zinc-800 bg-zinc-900/60 hover:border-zinc-700 transition-all space-y-2"
                >
                  <div className="flex items-start gap-2.5">
                    {/* Icon */}
                    <div
                      className="w-8 h-8 rounded flex items-center justify-center shrink-0 text-base shadow"
                      style={{ backgroundColor: ext.iconBg }}
                    >
                      {ext.iconSymbol}
                    </div>

                    {/* Metadata */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-1">
                        <span className="font-semibold text-zinc-100 truncate text-xs">
                          {ext.displayName}
                        </span>
                        {ext.wasmRuntime && (
                          <span className="text-[9px] px-1 py-0.2 bg-purple-950/80 text-purple-300 border border-purple-800/60 rounded font-mono" title="Native Wasm Sandboxed Runtime">
                            WASM
                          </span>
                        )}
                      </div>
                      <div className="text-[10px] text-zinc-500 truncate">
                        {ext.publisher} · {ext.version}
                      </div>
                    </div>
                  </div>

                  <p className="text-[11px] text-zinc-400 leading-snug line-clamp-2">
                    {ext.description}
                  </p>

                  {/* Actions & Rating */}
                  <div className="flex items-center justify-between pt-1 border-t border-zinc-800/80 text-[10px]">
                    <span className="text-zinc-500">
                      ★ {ext.rating} · {ext.downloads}
                    </span>

                    <div className="flex items-center gap-1">
                      {ext.installed ? (
                        <>
                          <button
                            type="button"
                            onClick={() => onToggleExtension(ext.id)}
                            className={`px-2 py-0.5 rounded font-medium transition-colors ${
                              ext.enabled
                                ? 'bg-zinc-800 hover:bg-zinc-700 text-zinc-200'
                                : 'bg-amber-900/30 text-amber-300 border border-amber-800'
                            }`}
                          >
                            {ext.enabled ? 'Disable' : 'Enable'}
                          </button>
                          <button
                            type="button"
                            onClick={() => onToggleInstallExtension(ext.id)}
                            className="px-2 py-0.5 rounded font-medium bg-red-950/40 hover:bg-red-900/50 text-red-300 border border-red-900/50 transition-colors"
                          >
                            Uninstall
                          </button>
                        </>
                      ) : (
                        <button
                          type="button"
                          onClick={() => onToggleInstallExtension(ext.id)}
                          className="px-2.5 py-0.5 rounded font-medium bg-orange-600 hover:bg-orange-500 text-white transition-colors"
                        >
                          Install (Rust Wasm)
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ================= PROFILER VIEW ================= */}
        {activeTab === 'profiler' && (
          <div className="p-3 space-y-4">
            <div className="p-3 rounded bg-zinc-900 border border-orange-500/30 space-y-2">
              <div className="flex items-center gap-2 text-orange-400 font-bold text-xs">
                <Zap className="w-4 h-4" />
                <span>Rust Native Performance Core</span>
              </div>
              <p className="text-[11px] text-zinc-400 leading-relaxed">
                Zero JavaScript garbage collection. Custom Ropey text buffers paired with wgpu GPU glyph streaming for 120 FPS latency.
              </p>
            </div>

            {/* Latency Comparison Card */}
            <div className="space-y-2">
              <div className="text-[11px] font-bold text-zinc-400 uppercase">TYPING LATENCY (LOWER IS BETTER)</div>
              
              <div className="space-y-1.5">
                <div>
                  <div className="flex justify-between text-[11px] text-orange-300 font-semibold mb-0.5">
                    <span>OxideCode (Pure Rust)</span>
                    <span className="font-mono">0.8 ms</span>
                  </div>
                  <div className="h-3 w-full bg-zinc-800 rounded overflow-hidden">
                    <div className="h-full bg-gradient-to-r from-orange-500 to-amber-400 rounded w-[4%]" />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-[11px] text-zinc-400 mb-0.5">
                    <span>Zed Editor (Rust)</span>
                    <span className="font-mono">2.1 ms</span>
                  </div>
                  <div className="h-3 w-full bg-zinc-800 rounded overflow-hidden">
                    <div className="h-full bg-zinc-500 rounded w-[9%]" />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-[11px] text-zinc-500 mb-0.5">
                    <span>Standard VS Code (Electron)</span>
                    <span className="font-mono">24.5 ms</span>
                  </div>
                  <div className="h-3 w-full bg-zinc-800 rounded overflow-hidden">
                    <div className="h-full bg-red-500/80 rounded w-[100%]" />
                  </div>
                </div>
              </div>
            </div>

            {/* Memory Comparison Card */}
            <div className="space-y-2">
              <div className="text-[11px] font-bold text-zinc-400 uppercase">RAM CONSUMPTION (IDLE)</div>
              
              <div className="grid grid-cols-2 gap-2 text-center">
                <div className="p-2 rounded bg-zinc-900 border border-emerald-900/40">
                  <div className="text-emerald-400 font-bold font-mono text-base">22 MB</div>
                  <div className="text-[10px] text-zinc-400">OxideCode Rust</div>
                </div>
                <div className="p-2 rounded bg-zinc-900 border border-red-900/40">
                  <div className="text-red-400 font-bold font-mono text-base">480 MB</div>
                  <div className="text-[10px] text-zinc-400">Electron VS Code</div>
                </div>
              </div>
            </div>

            <button
              type="button"
              onClick={() => onRunTerminalCommand('bench')}
              className="w-full py-1.5 bg-orange-600 hover:bg-orange-500 text-white rounded font-medium flex items-center justify-center gap-1.5 transition-colors"
            >
              <Zap className="w-3.5 h-3.5" />
              <span>Run Live Benchmark</span>
            </button>
          </div>
        )}

        {/* ================= DESIGN MOCKUPS VIEW ================= */}
        {activeTab === 'mockups' && (
          <div className="p-3 space-y-3">
            <div className="text-[11px] text-zinc-400">
              High-resolution generated desktop mockups created for this repository:
            </div>

            {/* Mockup 1: Desktop Shell */}
            <div
              onClick={() =>
                onOpenMockupModal(
                  '/src/assets/images/rust_vscode_desktop_mockup_1793221900017.jpg',
                  'OxideCode Desktop Environment (macOS Native Shell)'
                )
              }
              className="border border-zinc-700/80 rounded-lg overflow-hidden group cursor-pointer hover:border-orange-500 transition-all bg-zinc-900"
            >
              <div className="aspect-video w-full overflow-hidden bg-black/40">
                <img
                  src="/src/assets/images/rust_vscode_desktop_mockup_1793221900017.jpg"
                  alt="OxideCode Rust VS Code clone desktop mockup"
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-200"
                />
              </div>
              <div className="p-2 space-y-1">
                <div className="font-semibold text-zinc-200 flex items-center justify-between">
                  <span>1. Desktop IDE Shell</span>
                  <ExternalLink className="w-3 h-3 text-zinc-400 group-hover:text-orange-400" />
                </div>
                <p className="text-[10px] text-zinc-400">
                  Native macOS dark theme, 0.9ms latency HUD, Rust syntax highlighting, and integrated Cargo terminal.
                </p>
              </div>
            </div>

            {/* Mockup 2: Extensions & Split Benchmark */}
            <div
              onClick={() =>
                onOpenMockupModal(
                  '/src/assets/images/rust_ide_extensions_mockup_1793221915012.jpg',
                  'OxideCode Extensions Marketplace & Split Performance Benchmarks'
                )
              }
              className="border border-zinc-700/80 rounded-lg overflow-hidden group cursor-pointer hover:border-orange-500 transition-all bg-zinc-900"
            >
              <div className="aspect-video w-full overflow-hidden bg-black/40">
                <img
                  src="/src/assets/images/rust_ide_extensions_mockup_1793221915012.jpg"
                  alt="OxideCode extensions and latency graph mockup"
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-200"
                />
              </div>
              <div className="p-2 space-y-1">
                <div className="font-semibold text-zinc-200 flex items-center justify-between">
                  <span>2. Extensions &amp; Profiler</span>
                  <ExternalLink className="w-3 h-3 text-zinc-400 group-hover:text-orange-400" />
                </div>
                <p className="text-[10px] text-zinc-400">
                  Extensions Marketplace running rust-analyzer, GitLens, Copilot with live side-by-side benchmark graph.
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
