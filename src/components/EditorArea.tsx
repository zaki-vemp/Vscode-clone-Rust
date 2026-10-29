import React, { useState, useRef, useEffect } from 'react';
import {
  X,
  Columns,
  MoreHorizontal,
  ChevronRight,
  Lightbulb,
  Check,
  Copy,
  Sparkles,
  Command,
} from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';
import { WorkspaceFile, EditorSettings, OpenTab, Diagnostic } from '../types/editor.ts';

interface EditorAreaProps {
  theme: ThemeColors;
  settings: EditorSettings;
  files: WorkspaceFile[];
  openTabs: OpenTab[];
  activeFileId: string;
  onSelectTab: (fileId: string) => void;
  onCloseTab: (fileId: string) => void;
  onUpdateFileContent: (fileId: string, newContent: string) => void;
  onToggleSplit: () => void;
  isSplit: boolean;
  onRecordLatency: (ms: number) => void;
  onRunQuickFix?: (diagnostic: Diagnostic) => void;
}

export const EditorArea: React.FC<EditorAreaProps> = ({
  theme,
  settings,
  files,
  openTabs,
  activeFileId,
  onSelectTab,
  onCloseTab,
  onUpdateFileContent,
  onToggleSplit,
  isSplit,
  onRecordLatency,
  onRunQuickFix,
}) => {
  const activeFile = files.find((f) => f.id === activeFileId) || files[0];
  const [cursorLine, setCursorLine] = useState(16);
  const [cursorCol, setCursorCol] = useState(14);
  const [showAutocomplete, setShowAutocomplete] = useState(false);
  const [autocompleteFilter, setAutocompleteFilter] = useState('');
  const [copilotAccepted, setCopilotAccepted] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const lines = activeFile ? activeFile.content.split('\n') : [''];

  const getFileIcon = (fileName: string) => {
    if (fileName.endsWith('.rs')) return <span className="text-orange-500 font-bold text-xs select-none">🦀</span>;
    if (fileName.endsWith('.toml')) return <span className="text-amber-400 font-bold text-xs select-none">⚙️</span>;
    if (fileName.endsWith('.md')) return <span className="text-blue-400 font-bold text-xs select-none">📄</span>;
    return <span className="text-zinc-400 font-bold text-xs select-none">📁</span>;
  };

  // Autocomplete candidates for Rust
  const rustKeywords = [
    { label: 'println!', type: 'macro', detail: 'println!("{}", val)' },
    { label: 'Result<T, E>', type: 'type', detail: 'enum Result<T, E>' },
    { label: 'Option<T>', type: 'type', detail: 'enum Option<T>' },
    { label: 'RopeBuffer', type: 'struct', detail: 'crate::buffer::RopeBuffer' },
    { label: 'WgpuSurface', type: 'struct', detail: 'crate::renderer::WgpuSurface' },
    { label: 'Arc::new()', type: 'function', detail: 'std::sync::Arc' },
    { label: 'tokio::spawn()', type: 'function', detail: 'tokio::task' },
    { label: 'match', type: 'keyword', detail: 'Pattern matching' },
  ];

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    const startTime = performance.now();

    // Tab accepts Copilot suggestion or inserts 4 spaces
    if (e.key === 'Tab') {
      e.preventDefault();
      if (!copilotAccepted && settings.copilotSuggestions) {
        setCopilotAccepted(true);
        // append copilot snippet
        const updated = activeFile.content + '\n    // Copilot suggestion applied\n';
        onUpdateFileContent(activeFile.id, updated);
      } else {
        const textarea = textareaRef.current;
        if (textarea) {
          const start = textarea.selectionStart;
          const end = textarea.selectionEnd;
          const val = textarea.value;
          const newVal = val.substring(0, start) + '    ' + val.substring(end);
          onUpdateFileContent(activeFile.id, newVal);
          setTimeout(() => {
            textarea.selectionStart = textarea.selectionEnd = start + 4;
          }, 0);
        }
      }
    }

    // Ctrl+Space triggers autocomplete
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault();
      setShowAutocomplete(true);
    }

    // Measure key latency to emulate sub-millisecond Rust buffer dispatch
    requestAnimationFrame(() => {
      const delta = Math.max(0.4, Number((performance.now() - startTime).toFixed(2)));
      onRecordLatency(delta);
    });
  };

  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const start = performance.now();
    const val = e.target.value;
    onUpdateFileContent(activeFile.id, val);

    // Calculate line & col
    const selStart = e.target.selectionStart;
    const textBefore = val.substring(0, selStart);
    const lineNum = textBefore.split('\n').length;
    const colNum = textBefore.split('\n').pop()?.length || 1;
    setCursorLine(lineNum);
    setCursorCol(colNum);

    const delta = Math.max(0.5, Number((performance.now() - start).toFixed(2)));
    onRecordLatency(delta);
  };

  // Syntax colorizer helper for Rust tokens
  const renderHighlightedLine = (lineText: string, lineIndex: number) => {
    if (!lineText) return <span>&nbsp;</span>;

    // Check for comments
    if (lineText.trim().startsWith('//')) {
      return <span className="text-emerald-500/80 italic">{lineText}</span>;
    }

    // Split words while preserving whitespace
    const tokens = lineText.split(/(\s+|[(),;:{}[\]"'.<>!])/);

    return (
      <span>
        {tokens.map((token, i) => {
          if (!token) return null;

          // Keywords
          if (
            ['fn', 'pub', 'struct', 'impl', 'let', 'mut', 'use', 'mod', 'async', 'await', 'return', 'match', 'for', 'in', 'loop', 'while', 'if', 'else', 'crate'].includes(
              token
            )
          ) {
            return (
              <span key={i} className="text-[#c678dd] font-semibold">
                {token}
              </span>
            );
          }

          // Types
          if (
            ['String', 'usize', 'u64', 'u32', 'i32', 'bool', 'Result', 'Option', 'Ok', 'Err', 'Some', 'None', 'Arc', 'RwLock', 'Box', 'Self', 'EditorState', 'RopeBuffer', 'WgpuSurface', 'Error'].includes(
              token
            )
          ) {
            return (
              <span key={i} className="text-[#e5c07b]">
                {token}
              </span>
            );
          }

          // Macros
          if (token === 'println' || token === 'vec' || token === 'panic' || token === 'format') {
            return (
              <span key={i} className="text-[#61afef]">
                {token}
              </span>
            );
          }

          // String literals
          if (token.startsWith('"') || token.endsWith('"')) {
            return (
              <span key={i} className="text-[#98c379]">
                {token}
              </span>
            );
          }

          // Numbers
          if (/^\d+$/.test(token)) {
            return (
              <span key={i} className="text-[#d19a66]">
                {token}
              </span>
            );
          }

          return <span key={i}>{token}</span>;
        })}
      </span>
    );
  };

  return (
    <main
      className="flex-1 flex flex-col h-full overflow-hidden select-none relative"
      style={{
        backgroundColor: theme.editorBg,
        color: theme.textPrimary,
      }}
      aria-label="Editor Area"
    >
      {/* Tab Bar */}
      <div
        className="h-9 flex items-center justify-between border-b shrink-0 overflow-x-auto select-none"
        style={{
          backgroundColor: theme.tabInactiveBg,
          borderColor: theme.border,
        }}
      >
        <div className="flex items-center h-full">
          {openTabs.map((tab) => {
            const isActive = tab.fileId === activeFileId;
            return (
              <div
                key={tab.fileId}
                onClick={() => onSelectTab(tab.fileId)}
                className={`h-full px-3 flex items-center gap-2 border-r text-xs cursor-pointer group transition-colors relative ${
                  isActive
                    ? 'text-white border-t-2'
                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/30'
                }`}
                style={{
                  backgroundColor: isActive ? theme.tabActiveBg : 'transparent',
                  borderColor: theme.border,
                  borderTopColor: isActive ? theme.accent : 'transparent',
                }}
              >
                {getFileIcon(tab.title)}
                <span className="truncate max-w-[140px] font-medium">{tab.title}</span>

                {/* Dirty indicator or close button */}
                <div className="flex items-center">
                  {tab.isDirty ? (
                    <span className="w-2 h-2 rounded-full bg-white group-hover:hidden" />
                  ) : null}
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      onCloseTab(tab.fileId);
                    }}
                    className={`p-0.5 rounded hover:bg-zinc-700/60 text-zinc-400 hover:text-white ${
                      tab.isDirty ? 'hidden group-hover:block' : ''
                    }`}
                    title="Close Tab (Ctrl+W)"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              </div>
            );
          })}
        </div>

        {/* Tab Actions */}
        <div className="flex items-center px-2 space-x-1 text-zinc-400">
          <button
            type="button"
            onClick={onToggleSplit}
            className="p-1 hover:text-white rounded hover:bg-zinc-800"
            title="Split Editor Right (Ctrl+\)"
          >
            <Columns className="w-3.5 h-3.5" />
          </button>
          <button
            type="button"
            className="p-1 hover:text-white rounded hover:bg-zinc-800"
            title="More Tab Actions"
          >
            <MoreHorizontal className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Breadcrumbs Bar */}
      <div
        className="h-6 px-4 flex items-center text-[11px] border-b text-zinc-400 shrink-0 font-mono gap-1 select-none"
        style={{
          backgroundColor: theme.editorBg,
          borderColor: theme.border,
        }}
      >
        <span className="hover:text-zinc-200 cursor-pointer">oxide-editor</span>
        <ChevronRight className="w-3 h-3 text-zinc-600" />
        <span className="hover:text-zinc-200 cursor-pointer">
          {activeFile.path.split('/')[0]}
        </span>
        <ChevronRight className="w-3 h-3 text-zinc-600" />
        <span className="text-zinc-200 font-semibold flex items-center gap-1">
          {getFileIcon(activeFile.name)}
          {activeFile.name}
        </span>
        <ChevronRight className="w-3 h-3 text-zinc-600" />
        <span className="text-orange-400 font-semibold">
          {activeFile.language === 'rust' ? 'fn main()' : 'root'}
        </span>
      </div>

      {/* Main Split / Single Editor Surface */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Primary Editor Pane */}
        <div className="flex-1 flex overflow-hidden relative font-mono text-[13px] leading-5">
          {/* Gutter: Line Numbers & Git Status */}
          <div
            className="w-14 py-2 shrink-0 select-none text-right pr-3 space-y-0.5 border-r"
            style={{
              backgroundColor: theme.editorBg,
              borderColor: theme.border,
              color: theme.editorLineNumber,
            }}
          >
            {lines.map((_, idx) => {
              const lineNum = idx + 1;
              const isCurrent = lineNum === cursorLine;
              // Emulate git gutter
              const isAdded = lineNum === 16;
              const isModified = lineNum === 22;

              return (
                <div
                  key={idx}
                  className={`flex items-center justify-end gap-1 relative ${
                    isCurrent ? 'text-zinc-200 font-bold' : ''
                  }`}
                >
                  {/* Git Diff Indicator */}
                  {isAdded && (
                    <span className="absolute left-1 w-0.5 h-4 bg-emerald-500 rounded" />
                  )}
                  {isModified && (
                    <span className="absolute left-1 w-0.5 h-4 bg-blue-500 rounded" />
                  )}
                  <span>{lineNum}</span>
                </div>
              );
            })}
          </div>

          {/* Code Text Surface */}
          <div className="flex-1 relative overflow-auto py-2 px-3">
            {/* Visual Token Rendering Layer */}
            <div className="space-y-0.5 select-text pointer-events-none">
              {lines.map((line, idx) => {
                const lineNum = idx + 1;
                const isCurrent = lineNum === cursorLine;

                return (
                  <div
                    key={idx}
                    className={`flex items-center min-h-[20px] rounded px-1 transition-colors relative ${
                      isCurrent ? 'bg-zinc-800/40' : ''
                    }`}
                  >
                    {renderHighlightedLine(line, idx)}

                    {/* rust-analyzer Inlay Hints */}
                    {settings.inlayHints && lineNum === 16 && (
                      <span className="ml-2 px-1 py-0.2 text-[10px] bg-zinc-800/80 text-zinc-400 rounded border border-zinc-700/60 select-none">
                        : Editor
                      </span>
                    )}
                    {settings.inlayHints && lineNum === 22 && (
                      <span className="ml-2 px-1 py-0.2 text-[10px] bg-zinc-800/80 text-zinc-400 rounded border border-zinc-700/60 select-none">
                        : io::Result&lt;()&gt;
                      </span>
                    )}

                    {/* GitLens Blame Inline Annotation on Active Line */}
                    {settings.gitLensBlame && isCurrent && (
                      <span className="ml-6 text-[11px] text-zinc-500 italic select-none font-sans">
                        You, 2 hours ago • perf: zero-copy rope buffer commit e89a421
                      </span>
                    )}

                    {/* GitHub Copilot Inline Ghost Suggestion */}
                    {settings.copilotSuggestions && !copilotAccepted && isCurrent && lineNum === cursorLine && (
                      <span className="ml-2 text-zinc-500 select-none animate-pulse">
                        {'.inspect(|e| println!("render: {:?}", e))? '}
                        <span className="text-[10px] text-purple-400 font-sans border border-purple-800 px-1 rounded ml-1">[Tab]</span>
                      </span>
                    )}

                    {/* Quick Fix lightbulb on line with diagnostic */}
                    {lineNum === 16 && (
                      <div className="absolute right-4 pointer-events-auto">
                        <button
                          type="button"
                          onClick={() => {
                            if (onRunQuickFix) {
                              onRunQuickFix({
                                id: 'unused-mut',
                                fileId: activeFile.id,
                                filePath: activeFile.path,
                                line: 16,
                                column: 9,
                                message: 'variable does not need to be mutable: `let mut editor`',
                                severity: 'warning',
                                source: 'clippy',
                                quickFix: {
                                  title: 'Quick Fix: Remove `mut` keyword',
                                  action: 'remove_mut',
                                },
                              });
                            }
                          }}
                          className="p-1 rounded bg-amber-950/60 border border-amber-600/50 text-amber-400 hover:bg-amber-900/60 shadow"
                          title="Quick Fix: Remove unused `mut` keyword"
                        >
                          <Lightbulb className="w-3.5 h-3.5 animate-bounce" />
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>

            {/* Transparent Real Textarea for Keyboard Typing & Selection */}
            <textarea
              ref={textareaRef}
              value={activeFile.content}
              onChange={handleTextChange}
              onKeyDown={handleKeyDown}
              onClick={(e) => {
                const target = e.target as HTMLTextAreaElement;
                const textBefore = target.value.substring(0, target.selectionStart);
                setCursorLine(textBefore.split('\n').length);
                setCursorCol(textBefore.split('\n').pop()?.length || 1);
              }}
              spellCheck={false}
              autoCapitalize="off"
              autoComplete="off"
              className="absolute inset-0 w-full h-full opacity-0 cursor-text resize-none font-mono text-[13px] leading-5 p-2 outline-none z-10"
              aria-label="Code Editor Content"
            />
          </div>

          {/* Autocomplete Dropdown Popup */}
          {showAutocomplete && (
            <div
              className="absolute left-28 top-32 w-64 rounded-md shadow-2xl border p-1 text-xs z-50 animate-in fade-in zoom-in-95"
              style={{
                backgroundColor: theme.sidebarBg,
                borderColor: theme.border,
                color: theme.textPrimary,
              }}
            >
              <div className="px-2 py-1 text-[10px] text-zinc-500 font-bold uppercase border-b border-zinc-700/60 flex items-center justify-between">
                <span>rust-analyzer suggestions</span>
                <span className="text-orange-400">0.2ms</span>
              </div>
              <div className="py-1 space-y-0.5">
                {rustKeywords.map((item, idx) => (
                  <button
                    key={idx}
                    type="button"
                    onClick={() => {
                      onUpdateFileContent(activeFile.id, activeFile.content + item.label);
                      setShowAutocomplete(false);
                    }}
                    className="w-full text-left px-2 py-1 rounded flex items-center justify-between hover:bg-orange-600 hover:text-white transition-colors"
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-[10px] text-purple-400 font-mono">[{item.type}]</span>
                      <span className="font-semibold">{item.label}</span>
                    </div>
                    <span className="text-[10px] text-zinc-500 truncate max-w-[100px]">{item.detail}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Minimap on Right */}
          {settings.minimap && (
            <aside
              className="w-20 border-l shrink-0 overflow-hidden relative select-none hidden lg:block opacity-75 hover:opacity-100 transition-opacity"
              style={{
                backgroundColor: theme.editorBg,
                borderColor: theme.border,
              }}
              aria-label="Minimap"
            >
              <div className="scale-[0.25] origin-top-left p-2 space-y-1 w-80 font-mono text-zinc-500 leading-3">
                {lines.map((line, idx) => (
                  <div key={idx} className="truncate">
                    {line}
                  </div>
                ))}
              </div>
              {/* Draggable Viewport Slider */}
              <div
                className="absolute top-2 left-0 right-0 h-16 bg-zinc-700/30 border border-zinc-500/40 rounded cursor-grab active:cursor-grabbing"
                title="Current Viewport"
              />
            </aside>
          )}
        </div>

        {/* Optional Secondary Split Pane (Side-by-Side) */}
        {isSplit && (
          <div
            className="flex-1 flex flex-col border-l font-mono text-xs overflow-hidden"
            style={{
              borderColor: theme.border,
              backgroundColor: theme.editorBg,
            }}
          >
            <div
              className="h-8 px-3 flex items-center justify-between border-b text-[11px] text-zinc-400 select-none shrink-0"
              style={{
                backgroundColor: theme.tabInactiveBg,
                borderColor: theme.border,
              }}
            >
              <div className="flex items-center gap-1.5 font-semibold text-zinc-300">
                <span className="text-amber-400">⚙️</span>
                <span>Cargo.toml (Split View)</span>
              </div>
              <span className="text-[10px] text-orange-400 font-mono">Rust Engine</span>
            </div>

            <div className="p-3 overflow-auto text-zinc-300 space-y-1">
              <pre className="text-xs font-mono">
                {files.find((f) => f.name === 'Cargo.toml')?.content || ''}
              </pre>
            </div>
          </div>
        )}
      </div>
    </main>
  );
};
