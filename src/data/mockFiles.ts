import { WorkspaceFile } from '../types/editor.ts';

export const INITIAL_FILES: WorkspaceFile[] = [
  {
    id: 'f-main-rs',
    name: 'main.rs',
    path: 'src/main.rs',
    language: 'rust',
    content: `//! OxideCode: Ultra-low latency, pure Rust native desktop clone of VS Code.
//! Powered by wgpu GPU rendering, custom Ropey text buffers, and WASM extension host.

use std::sync::Arc;
use tokio::sync::RwLock;

mod buffer;
mod extensions;
mod lsp;
mod renderer;

use buffer::RopeBuffer;
use renderer::WgpuSurface;

#[derive(Debug, Clone)]
pub struct EditorState {
    pub active_file: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub typing_latency_us: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Initializing OxideCode Native Core v0.1.0...");

    // 1. Initialize wgpu hardware-accelerated pipeline (120 FPS target)
    let renderer = WgpuSurface::new().await?;
    println!("✓ WGPU Backend initialized (DirectX 12 / Metal / Vulkan)");

    // 2. Load chunked zero-copy RopeBuffer
    let buffer = Arc::new(RwLock::new(RopeBuffer::from_str("src/main.rs")));
    println!("✓ Text buffer ready. Memory footprint: 18.4 MB");

    // 3. Spawn rust-analyzer LSP daemon
    lsp::spawn_rust_analyzer().await?;
    println!("✓ rust-analyzer LSP server ready (sub-millisecond diagnostics)");

    // 4. Initialize WebAssembly VS Code Extension Sandbox
    extensions::init_wasm_host().await?;
    println!("✓ Extension runtime loaded (GitLens, Prettier, Copilot enabled)");

    // Main event loop with zero GC pauses
    let mut event_pump = renderer.event_loop();
    while let Some(event) = event_pump.next().await {
        renderer.handle_event(event, &buffer).await;
    }

    Ok(())
}`,
    isModified: true,
  },
  {
    id: 'f-rope-rs',
    name: 'rope.rs',
    path: 'src/buffer/rope.rs',
    language: 'rust',
    content: `use std::fmt;

/// High performance chunked Rope buffer for sub-millisecond keyboard response.
/// Avoids the memory fragmentation and GC pauses of browser DOM/V8 string slicing.
pub struct RopeBuffer {
    chunks: Vec<String>,
    line_count: usize,
    byte_count: usize,
}

impl RopeBuffer {
    pub fn new() -> Self {
        Self {
            chunks: vec![String::new()],
            line_count: 1,
            byte_count: 0,
        }
    }

    pub fn from_str(initial: &str) -> Self {
        let chunks: Vec<String> = initial.lines().map(String::from).collect();
        let line_count = chunks.len().max(1);
        let byte_count = initial.len();
        Self { chunks, line_count, byte_count }
    }

    #[inline(always)]
    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        if line < self.chunks.len() {
            let chunk = &mut self.chunks[line];
            if col <= chunk.len() {
                chunk.insert(col, ch);
                self.byte_count += ch.len_utf8();
            }
        }
    }

    pub fn line(&self, idx: usize) -> Option<&str> {
        self.chunks.get(idx).map(|s| s.as_str())
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }
}`,
  },
  {
    id: 'f-cargo-toml',
    name: 'Cargo.toml',
    path: 'Cargo.toml',
    language: 'toml',
    content: `[package]
name = "oxide-code"
version = "0.1.0"
edition = "2021"
authors = ["OxideCode Core Team <core@oxidecode.dev>"]
description = "High-performance Rust native clone of VS Code"
license = "MIT OR Apache-2.0"

[dependencies]
wgpu = "0.20.0"
winit = "0.29.15"
tokio = { version = "1.37.0", features = ["full"] }
tree-sitter = "0.22.2"
tree-sitter-rust = "0.21.0"
wasmtime = "18.0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rayon = "1.10.0"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true`,
    isModified: true,
  },
  {
    id: 'f-host-rs',
    name: 'host.rs',
    path: 'src/extensions/host.rs',
    language: 'rust',
    content: `use wasmtime::*;
use std::collections::HashMap;

/// Sandboxed Wasm extension host running VS Code extension JS/Wasm bundles.
/// Bridges VS Code API calls (vscode.window, vscode.workspace, vscode.languages)
/// directly to Rust native IPC primitives with zero JS main-thread blockage.
pub struct ExtensionSandbox {
    engine: Engine,
    store: Store<()>,
    active_extensions: HashMap<String, String>,
}

impl ExtensionSandbox {
    pub fn new() -> Result<Self, Error> {
        let mut config = Config::new();
        config.wasm_simd(true);
        config.cranelift_opt_level(OptLevel::Speed);

        let engine = Engine::new(&config)?;
        let store = Store::new(&engine, ());

        Ok(Self {
            engine,
            store,
            active_extensions: HashMap::new(),
        })
    }

    pub fn load_extension(&mut self, ext_id: &str, wasm_bytes: &[u8]) -> Result<(), Error> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        println!("🚀 Loaded sandboxed extension [{}] via Cranelift JIT", ext_id);
        self.active_extensions.insert(ext_id.to_string(), "active".into());
        Ok(())
    }
}`,
  },
  {
    id: 'f-readme',
    name: 'README.md',
    path: 'README.md',
    language: 'markdown',
    content: `# OxideCode (🦀 Rust-Native VS Code Clone)

OxideCode is a desktop code editor engineered from the ground up in pure Rust. It maintains 100% user interface fidelity with Visual Studio Code while solving Electron's memory bloat, high typing latency, and GC stutter.

## Why Rust for VS Code?

| Metric | VS Code (Electron/Chromium) | OxideCode (Pure Rust + WGPU) |
|---|---|---|
| **Key-to-Glass Latency** | ~24.5 ms | **0.8 ms** (30x faster) |
| **Idle Memory Footprint** | ~480 MB | **22 MB** (22x leaner) |
| **Cold Startup Time** | ~1,250 ms | **42 ms** (instant launch) |
| **Render Target** | Chromium DOM / Canvas | **Direct GPU (Vulkan/Metal/DX12)** |
| **Extension Isolation** | Node.js process IPC | **Sandboxed Wasmtime VM** |

## VS Code Extension Compatibility
- **rust-analyzer**: First-class language server with real-time inlay hints & borrow diagnostics.
- **GitLens**: Supercharged git blame annotations, visual commit graph, and stash explorer.
- **GitHub Copilot**: Rust-native local token cache with instantaneous ghost-text suggestions.
- **Prettier / rustfmt**: Zero-latency deterministic code formatting.
- **Theme Engine**: Support for Tokyo Night, Catppuccin, Monokai Pro, and VS Code Dark Modern.`,
  },
];
