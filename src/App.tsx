/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useState, useEffect, useCallback } from 'react';
import { TitleBar } from './components/TitleBar.tsx';
import { ActivityBar, ActivityTab } from './components/ActivityBar.tsx';
import { SideBar } from './components/SideBar.tsx';
import { EditorArea } from './components/EditorArea.tsx';
import { BottomPanel } from './components/BottomPanel.tsx';
import { StatusBar } from './components/StatusBar.tsx';
import { CommandPalette } from './components/CommandPalette.tsx';
import { RustProfilerModal } from './components/RustProfilerModal.tsx';
import { MockupShowcaseModal } from './components/MockupShowcaseModal.tsx';

import { THEMES } from './styles/themes.ts';
import { INITIAL_FILES } from './data/mockFiles.ts';
import { INITIAL_EXTENSIONS } from './data/mockExtensions.ts';
import {
  WorkspaceFile,
  OpenTab,
  Diagnostic,
  EditorSettings,
  ExtensionItem,
} from './types/editor.ts';

export default function App() {
  // Application Settings
  const [settings, setSettings] = useState<EditorSettings>({
    theme: 'rust-oxide',
    inlayHints: true,
    formatOnSave: true,
    minimap: true,
    gitLensBlame: true,
    copilotSuggestions: true,
    fontSize: 13,
    platformTitlebar: 'macos',
    rustTelemetryHud: true,
    lineNumbers: true,
    tabSize: 4,
  });

  const theme = THEMES[settings.theme];

  // Workspace Files
  const [files, setFiles] = useState<WorkspaceFile[]>(INITIAL_FILES);
  const [activeFileId, setActiveFileId] = useState<string>('f-main-rs');

  // Open Editor Tabs
  const [openTabs, setOpenTabs] = useState<OpenTab[]>([
    { fileId: 'f-main-rs', title: 'main.rs', path: 'src/main.rs', isDirty: true },
    { fileId: 'f-cargo-toml', title: 'Cargo.toml', path: 'Cargo.toml', isDirty: false },
    { fileId: 'f-readme', title: 'README.md', path: 'README.md', isDirty: false },
  ]);

  // Extensions
  const [extensions, setExtensions] = useState<ExtensionItem[]>(INITIAL_EXTENSIONS);

  // Layout state
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [activeActivityTab, setActiveActivityTab] = useState<ActivityTab>('explorer');
  const [bottomPanelOpen, setBottomPanelOpen] = useState(true);
  const [isEditorSplit, setIsEditorSplit] = useState(false);

  // Modals state
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [rustProfilerOpen, setRustProfilerOpen] = useState(false);
  const [mockupModalOpen, setMockupModalOpen] = useState(false);
  const [selectedMockupImage, setSelectedMockupImage] = useState(
    '/src/assets/images/rust_vscode_desktop_mockup_1793221900017.jpg'
  );
  const [selectedMockupTitle, setSelectedMockupTitle] = useState(
    'OxideCode Desktop Environment (macOS Native Shell)'
  );

  // Terminal triggers
  const [terminalCommandToRun, setTerminalCommandToRun] = useState<string | undefined>();

  // Live Latency & Telemetry
  const [currentLatencyMs, setCurrentLatencyMs] = useState(0.8);

  // Diagnostics (rust-analyzer & clippy)
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([
    {
      id: 'unused-mut',
      fileId: 'f-main-rs',
      filePath: 'src/main.rs',
      line: 16,
      column: 9,
      message: 'variable does not need to be mutable: `let mut editor`',
      severity: 'warning',
      source: 'clippy',
      codeRule: 'unused_mut',
      quickFix: {
        title: 'Remove unused `mut` keyword',
        action: 'remove_mut',
      },
    },
  ]);

  const activeFile = files.find((f) => f.id === activeFileId) || files[0];

  // Update Settings helper
  const handleUpdateSettings = (newSettings: Partial<EditorSettings>) => {
    setSettings((prev) => ({ ...prev, ...newSettings }));
  };

  // Select a file
  const handleSelectFile = (fileId: string) => {
    setActiveFileId(fileId);
    const file = files.find((f) => f.id === fileId);
    if (file && !openTabs.some((t) => t.fileId === fileId)) {
      setOpenTabs((prev) => [
        ...prev,
        { fileId: file.id, title: file.name, path: file.path, isDirty: file.isModified },
      ]);
    }
  };

  // Close Tab
  const handleCloseTab = (fileId: string) => {
    const updated = openTabs.filter((t) => t.fileId !== fileId);
    setOpenTabs(updated);
    if (activeFileId === fileId && updated.length > 0) {
      setActiveFileId(updated[updated.length - 1].fileId);
    }
  };

  // Update File Content
  const handleUpdateFileContent = (fileId: string, newContent: string) => {
    setFiles((prev) =>
      prev.map((f) =>
        f.id === fileId ? { ...f, content: newContent, isModified: true } : f
      )
    );
    setOpenTabs((prev) =>
      prev.map((t) => (t.fileId === fileId ? { ...t, isDirty: true } : t))
    );
  };

  // Create new file
  const handleCreateFile = (fileName: string) => {
    const newFile: WorkspaceFile = {
      id: `f-${Date.now()}`,
      name: fileName,
      path: `src/${fileName}`,
      language: fileName.endsWith('.rs')
        ? 'rust'
        : fileName.endsWith('.toml')
        ? 'toml'
        : 'markdown',
      content: `// New file: ${fileName}\n\npub fn init() {\n    println!("Initialized ${fileName}");\n}\n`,
      isNew: true,
    };
    setFiles((prev) => [...prev, newFile]);
    handleSelectFile(newFile.id);
  };

  // Delete file
  const handleDeleteFile = (fileId: string) => {
    setFiles((prev) => prev.filter((f) => f.id !== fileId));
    handleCloseTab(fileId);
  };

  // Toggle Extension enabled state
  const handleToggleExtension = (extId: string) => {
    setExtensions((prev) =>
      prev.map((ext) =>
        ext.id === extId ? { ...ext, enabled: !ext.enabled } : ext
      )
    );
  };

  // Toggle Extension install/uninstall
  const handleToggleInstallExtension = (extId: string) => {
    setExtensions((prev) =>
      prev.map((ext) =>
        ext.id === extId
          ? { ...ext, installed: !ext.installed, enabled: !ext.installed }
          : ext
      )
    );
  };

  // Execute command in terminal
  const handleRunTerminalCommand = (cmd: string) => {
    setBottomPanelOpen(true);
    setTerminalCommandToRun(cmd);
  };

  // Quick fix resolution
  const handleRunQuickFix = (diag: Diagnostic) => {
    if (diag.id === 'unused-mut') {
      const file = files.find((f) => f.id === diag.fileId);
      if (file) {
        const updated = file.content.replace(
          'let mut editor = Editor::new();',
          'let editor = Editor::new();'
        );
        handleUpdateFileContent(file.id, updated);
        setDiagnostics((prev) => prev.filter((d) => d.id !== diag.id));
      }
    }
  };

  // Open Mockup modal
  const handleOpenMockupModal = (src: string, title: string) => {
    setSelectedMockupImage(src);
    setSelectedMockupTitle(title);
    setMockupModalOpen(true);
  };

  // Global Keyboard Shortcuts
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Shift+P / Cmd+Shift+P -> Command Palette
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.code === 'KeyP') {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
      // Ctrl+P / Cmd+P -> Quick Open
      else if ((e.ctrlKey || e.metaKey) && e.code === 'KeyP' && !e.shiftKey) {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
      // Ctrl+B / Cmd+B -> Toggle Sidebar
      else if ((e.ctrlKey || e.metaKey) && e.code === 'KeyB') {
        e.preventDefault();
        setSidebarOpen((prev) => !prev);
      }
      // Ctrl+J / Cmd+J -> Toggle Bottom Panel
      else if ((e.ctrlKey || e.metaKey) && e.code === 'KeyJ') {
        e.preventDefault();
        setBottomPanelOpen((prev) => !prev);
      }
      // Ctrl+\ / Cmd+\ -> Split Editor
      else if ((e.ctrlKey || e.metaKey) && e.code === 'Backslash') {
        e.preventDefault();
        setIsEditorSplit((prev) => !prev);
      }
      // Ctrl+S / Cmd+S -> Save & Run Check
      else if ((e.ctrlKey || e.metaKey) && e.code === 'KeyS') {
        e.preventDefault();
        handleRunTerminalCommand('cargo check');
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  return (
    <div
      className="w-screen h-screen flex flex-col overflow-hidden font-sans select-none"
      style={{
        backgroundColor: theme.windowBg,
        color: theme.textPrimary,
      }}
    >
      {/* 1. Window TitleBar */}
      <TitleBar
        theme={theme}
        settings={settings}
        activeFileName={activeFile?.path || 'src/main.rs'}
        onOpenCommandPalette={() => setCommandPaletteOpen(true)}
        onOpenQuickOpen={() => setCommandPaletteOpen(true)}
        onToggleSidebar={() => setSidebarOpen(!sidebarOpen)}
        onToggleBottomPanel={() => setBottomPanelOpen(!bottomPanelOpen)}
        onToggleSplitEditor={() => setIsEditorSplit(!isEditorSplit)}
        onOpenRustProfiler={() => setRustProfilerOpen(true)}
        onExecuteCommand={handleRunTerminalCommand}
      />

      {/* 2. Main Workbench Body (ActivityBar + SideBar + Editor + BottomPanel) */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Activity Bar */}
        <ActivityBar
          theme={theme}
          activeTab={activeActivityTab}
          onSelectTab={(tab) => {
            if (activeActivityTab === tab && sidebarOpen) {
              setSidebarOpen(false);
            } else {
              setActiveActivityTab(tab);
              setSidebarOpen(true);
            }
          }}
          sidebarOpen={sidebarOpen}
          settings={settings}
          onUpdateSettings={handleUpdateSettings}
          modifiedFilesCount={files.filter((f) => f.isModified).length}
        />

        {/* Side Bar (Collapsible) */}
        {sidebarOpen && (
          <SideBar
            theme={theme}
            activeTab={activeActivityTab}
            files={files}
            activeFileId={activeFileId}
            onSelectFile={handleSelectFile}
            onCreateFile={handleCreateFile}
            onDeleteFile={handleDeleteFile}
            extensions={extensions}
            onToggleExtension={handleToggleExtension}
            onToggleInstallExtension={handleToggleInstallExtension}
            settings={settings}
            onUpdateSettings={handleUpdateSettings}
            onRunTerminalCommand={handleRunTerminalCommand}
            onOpenMockupModal={handleOpenMockupModal}
          />
        )}

        {/* Center: Editor Area + Bottom Panel */}
        <div className="flex-1 flex flex-col overflow-hidden relative">
          {/* Code Editor */}
          <EditorArea
            theme={theme}
            settings={settings}
            files={files}
            openTabs={openTabs}
            activeFileId={activeFileId}
            onSelectTab={handleSelectFile}
            onCloseTab={handleCloseTab}
            onUpdateFileContent={handleUpdateFileContent}
            onToggleSplit={() => setIsEditorSplit(!isEditorSplit)}
            isSplit={isEditorSplit}
            onRecordLatency={setCurrentLatencyMs}
            onRunQuickFix={handleRunQuickFix}
          />

          {/* Bottom Integrated Panel (Terminal, Diagnostics, Logs) */}
          <BottomPanel
            theme={theme}
            isOpen={bottomPanelOpen}
            onClose={() => setBottomPanelOpen(false)}
            diagnostics={diagnostics}
            onSelectDiagnostic={(diag) => {
              handleSelectFile(diag.fileId);
            }}
            onRunQuickFix={handleRunQuickFix}
            terminalCommandToRun={terminalCommandToRun}
            onClearRunCommand={() => setTerminalCommandToRun(undefined)}
          />
        </div>
      </div>

      {/* 3. Status Bar */}
      <StatusBar
        theme={theme}
        settings={settings}
        activeLanguage={activeFile?.language || 'rust'}
        cursorLine={16}
        cursorCol={14}
        errorsCount={0}
        warningsCount={diagnostics.length}
        currentLatencyMs={currentLatencyMs}
        onOpenRustProfiler={() => setRustProfilerOpen(true)}
        onOpenCommandPalette={() => setCommandPaletteOpen(true)}
      />

      {/* 4. Modals */}
      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        theme={theme}
        settings={settings}
        onUpdateSettings={handleUpdateSettings}
        onRunTerminalCommand={handleRunTerminalCommand}
        onOpenRustProfiler={() => setRustProfilerOpen(true)}
        onOpenMockupModal={handleOpenMockupModal}
        onToggleSplit={() => setIsEditorSplit(!isEditorSplit)}
        onSelectFile={handleSelectFile}
      />

      <RustProfilerModal
        isOpen={rustProfilerOpen}
        onClose={() => setRustProfilerOpen(false)}
        theme={theme}
        currentLatencyMs={currentLatencyMs}
        onRunTerminalCommand={handleRunTerminalCommand}
      />

      <MockupShowcaseModal
        isOpen={mockupModalOpen}
        onClose={() => setMockupModalOpen(false)}
        theme={theme}
        selectedImage={selectedMockupImage}
        selectedTitle={selectedMockupTitle}
        onSelectAnotherMockup={(src, title) => {
          setSelectedMockupImage(src);
          setSelectedMockupTitle(title);
        }}
      />
    </div>
  );
}
