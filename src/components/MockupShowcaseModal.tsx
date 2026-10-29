import React from 'react';
import { X, ExternalLink, Download, Check, Sparkles, Layers } from 'lucide-react';
import { ThemeColors } from '../styles/themes.ts';

interface MockupShowcaseModalProps {
  isOpen: boolean;
  onClose: () => void;
  theme: ThemeColors;
  selectedImage: string;
  selectedTitle: string;
  onSelectAnotherMockup: (src: string, title: string) => void;
}

export const MockupShowcaseModal: React.FC<MockupShowcaseModalProps> = ({
  isOpen,
  onClose,
  theme,
  selectedImage,
  selectedTitle,
  onSelectAnotherMockup,
}) => {
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
    }
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const mockups = [
    {
      src: '/src/assets/images/rust_vscode_desktop_mockup_1793221900017.jpg',
      title: 'OxideCode Desktop Environment (macOS Native Shell)',
      subtitle: 'Complete VS Code UI layout rewritten in pure Rust with wgpu GPU rendering',
      specs: [
        'Native macOS titlebar with traffic light window controls',
        'VS Code Activity Bar with Explorer, Git, Extensions, and Profiler icons',
        'Rust cargo project tree explorer (src/main.rs, Cargo.toml, README.md)',
        'Monaco-style syntax highlighting with rust-analyzer parameter inlay hints',
        'Integrated Cargo terminal running `cargo check` in 0.12s',
        'Live status bar indicating 0.9ms latency and 24MB RAM consumption',
      ],
    },
    {
      src: '/src/assets/images/rust_ide_extensions_mockup_1793221915012.jpg',
      title: 'Extensions Marketplace & Live Split Performance Benchmark',
      subtitle: 'Native VS Code extension ecosystem running in sandboxed WebAssembly runtime',
      specs: [
        'Extensions Marketplace featuring rust-analyzer, GitLens, Copilot, Prettier',
        'Fast Rust Wasm runtime badge on all verified extensions',
        'Side-by-side split editor displaying live performance benchmark graph',
        '0.8ms OxideCode typing latency vs 24ms standard Electron VS Code',
        '22MB RAM idle footprint vs 480MB Electron memory bloat',
        'Interactive install / uninstall and enable / disable extension management',
      ],
    },
  ];

  const currentMockup =
    mockups.find((m) => m.src === selectedImage) || mockups[0];

  return (
    <div
      className="fixed inset-0 bg-black/85 z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
      onClick={onClose}
    >
      <div
        className="w-full max-w-5xl rounded-xl shadow-2xl border overflow-hidden flex flex-col max-h-[92vh] animate-in zoom-in-95 duration-150 text-xs"
        style={{
          backgroundColor: theme.sidebarBg,
          borderColor: theme.border,
          color: theme.textPrimary,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="p-3.5 border-b flex items-center justify-between shrink-0"
          style={{ borderColor: theme.border }}
        >
          <div className="flex items-center gap-2">
            <span className="text-amber-400 font-bold text-base">🖼️</span>
            <div>
              <h2 className="font-bold text-sm text-zinc-100">{currentMockup.title}</h2>
              <p className="text-[11px] text-zinc-400">{currentMockup.subtitle}</p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Switch Mockup Tabs */}
            <div className="flex items-center gap-1 p-0.5 bg-zinc-900 rounded border border-zinc-800 mr-2">
              {mockups.map((m, idx) => (
                <button
                  key={idx}
                  type="button"
                  onClick={() => onSelectAnotherMockup(m.src, m.title)}
                  className={`px-2.5 py-1 rounded text-[11px] font-medium transition-colors ${
                    currentMockup.src === m.src
                      ? 'bg-orange-600 text-white'
                      : 'text-zinc-400 hover:text-white'
                  }`}
                >
                  Mockup {idx + 1}
                </button>
              ))}
            </div>

            <button
              type="button"
              onClick={onClose}
              className="p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-4 flex flex-col lg:flex-row gap-5">
          {/* Main High-Res Mockup Image */}
          <div className="flex-1 rounded-lg border border-zinc-700/80 overflow-hidden bg-black flex items-center justify-center relative group">
            <img
              src={currentMockup.src}
              alt={currentMockup.title}
              className="w-full h-auto object-contain max-h-[65vh] rounded"
            />
            <div className="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity">
              <a
                href={currentMockup.src}
                target="_blank"
                rel="noreferrer"
                className="px-2.5 py-1 bg-black/80 hover:bg-black text-white text-[10px] rounded border border-zinc-600 flex items-center gap-1 backdrop-blur"
              >
                <ExternalLink className="w-3 h-3" />
                <span>Full Resolution</span>
              </a>
            </div>
          </div>

          {/* Design Specs & Architecture Notes */}
          <div className="w-full lg:w-80 flex flex-col justify-between space-y-4">
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-orange-400 font-bold text-xs uppercase tracking-wider">
                <Sparkles className="w-4 h-4" />
                <span>Engineering &amp; UI Highlights</span>
              </div>

              <div className="space-y-2">
                {currentMockup.specs.map((spec, i) => (
                  <div key={i} className="flex items-start gap-2 text-zinc-300 text-[11px] leading-relaxed">
                    <Check className="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
                    <span>{spec}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* Performance Callout */}
            <div className="p-3 rounded-lg bg-zinc-900 border border-orange-500/30 space-y-1.5">
              <div className="flex items-center justify-between text-zinc-200 font-semibold text-[11px]">
                <span>Rust Native Target</span>
                <span className="text-orange-400 font-mono">120 FPS</span>
              </div>
              <p className="text-[10px] text-zinc-400 leading-normal">
                Generated as part of the OxideCode design specification for high-performance desktop development on macOS, Linux, and Windows.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
