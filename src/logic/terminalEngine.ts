/**
 * Terminal command runner and Rust toolchain simulator.
 * Supports: cargo run, cargo test, cargo clippy, cargo check, rustc --version, git status, git log, clear, help, bench.
 */

export interface TerminalOutputLine {
  id: string;
  type: 'input' | 'output' | 'error' | 'success' | 'info';
  text: string;
}

export interface TerminalResult {
  lines: TerminalOutputLine[];
  clear?: boolean;
}

export function executeCommand(cmd: string, currentBranch: string = 'main'): TerminalResult {
  const trimmed = cmd.trim();
  const idPrefix = Math.random().toString(36).substring(2, 8);

  if (!trimmed) {
    return { lines: [] };
  }

  if (trimmed === 'clear') {
    return { lines: [], clear: true };
  }

  if (trimmed === 'help') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: 'OxideCode Rust Shell v0.1.0 (wgpu native engine)' },
        { id: `${idPrefix}-2`, type: 'output', text: 'Available commands:' },
        { id: `${idPrefix}-3`, type: 'output', text: '  cargo check           - Run fast typecheck via rust-analyzer' },
        { id: `${idPrefix}-4`, type: 'output', text: '  cargo test            - Run Rust unit and integration test suite' },
        { id: `${idPrefix}-5`, type: 'output', text: '  cargo clippy          - Run linter warnings and borrow checker' },
        { id: `${idPrefix}-6`, type: 'output', text: '  cargo run             - Build and execute native OxideCode binary' },
        { id: `${idPrefix}-7`, type: 'output', text: '  cargo build --release - Compile optimized native binary' },
        { id: `${idPrefix}-8`, type: 'output', text: '  rustc --version       - Display installed Rust compiler' },
        { id: `${idPrefix}-9`, type: 'output', text: '  bench                 - Run microbenchmark comparing Rust vs Electron' },
        { id: `${idPrefix}-10`, type: 'output', text: '  git status            - Show working tree status' },
        { id: `${idPrefix}-11`, type: 'output', text: '  git log               - Show recent commits' },
        { id: `${idPrefix}-12`, type: 'output', text: '  clear                 - Clear terminal screen' },
      ],
    };
  }

  if (trimmed === 'rustc --version' || trimmed === 'rustc -V') {
    return {
      lines: [
        {
          id: `${idPrefix}-1`,
          type: 'output',
          text: 'rustc 1.82.0 (f6e511eec 2024-10-15) (OxideCode native toolchain x86_64-unknown-linux-gnu)',
        },
      ],
    };
  }

  if (trimmed === 'cargo check') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '    Checking oxide_core v0.1.0 (/workspace/oxide-editor)' },
        { id: `${idPrefix}-2`, type: 'info', text: '    Checking oxide_lsp v0.1.0 (/workspace/oxide-editor)' },
        { id: `${idPrefix}-3`, type: 'info', text: '    Checking oxide_gui v0.1.0 (/workspace/oxide-editor)' },
        {
          id: `${idPrefix}-4`,
          type: 'success',
          text: '    Finished `dev` profile [unoptimized + debuginfo] in 0.12s ⚡ (cache hit)',
        },
      ],
    };
  }

  if (trimmed === 'cargo test') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '   Compiling oxide_core v0.1.0 (/workspace/oxide-editor)' },
        { id: `${idPrefix}-2`, type: 'info', text: '    Finished test [unoptimized + debuginfo] in 0.38s' },
        { id: `${idPrefix}-3`, type: 'output', text: '     Running unittests src/lib.rs (target/debug/deps/oxide-61f2)' },
        { id: `${idPrefix}-4`, type: 'success', text: 'test buffer::tests::test_rope_insert ... ok' },
        { id: `${idPrefix}-5`, type: 'success', text: 'test buffer::tests::test_sub_millisecond_latency ... ok' },
        { id: `${idPrefix}-6`, type: 'success', text: 'test lsp::tests::test_rust_analyzer_protocol ... ok' },
        { id: `${idPrefix}-7`, type: 'success', text: 'test renderer::tests::test_wgpu_glyph_cache ... ok' },
        { id: `${idPrefix}-8`, type: 'success', text: 'test extensions::tests::test_wasm_sandbox_isolation ... ok' },
        { id: `${idPrefix}-9`, type: 'output', text: 'test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s' },
      ],
    };
  }

  if (trimmed === 'cargo clippy') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '    Checking oxide_core v0.1.0 (/workspace/oxide-editor)' },
        {
          id: `${idPrefix}-2`,
          type: 'output',
          text: 'warning: variable does not need to be mutable',
        },
        {
          id: `${idPrefix}-3`,
          type: 'output',
          text: '  --> src/main.rs:16:9\n   |\n16 |     let mut editor = Editor::new();\n   |         ----^^^^^^\n   |         help: remove `mut`\n   |\n   = note: `#[warn(unused_mut)]` on by default',
        },
        {
          id: `${idPrefix}-4`,
          type: 'success',
          text: '    Finished dev [unoptimized + debuginfo] in 0.22s (1 warning emitted)',
        },
      ],
    };
  }

  if (trimmed === 'cargo run' || trimmed === 'cargo r') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '    Finished dev [unoptimized + debuginfo] in 0.08s' },
        { id: `${idPrefix}-2`, type: 'output', text: '     Running `target/debug/oxide-editor`' },
        { id: `${idPrefix}-3`, type: 'success', text: '[INFO oxide_core] WGPU Backend: Vulkan/Metal hardware acceleration' },
        { id: `${idPrefix}-4`, type: 'success', text: '[INFO oxide_core] Tree-Sitter incremental parser initialized in 1.4ms' },
        { id: `${idPrefix}-5`, type: 'success', text: '[INFO oxide_core] Memory usage: 18.4 MB (0.0% swap)' },
        { id: `${idPrefix}-6`, type: 'output', text: 'OxideCode native window spawned successfully (120 FPS)' },
      ],
    };
  }

  if (trimmed === 'cargo build --release' || trimmed === 'cargo build -r') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '   Compiling oxide_core v0.1.0 (/workspace/oxide-editor)' },
        { id: `${idPrefix}-2`, type: 'info', text: '    Finished release [optimized + lto] in 2.14s' },
        { id: `${idPrefix}-3`, type: 'success', text: 'Binary created at: target/release/oxide (Size: 14.2 MB stripped)' },
      ],
    };
  }

  if (trimmed === 'bench') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: '=== Microbenchmark: OxideCode (Rust) vs Electron VS Code ===' },
        { id: `${idPrefix}-2`, type: 'success', text: 'Typing Key-to-Glass Latency:  OxideCode: 0.8ms   | VS Code: 24.5ms  (30x faster)' },
        { id: `${idPrefix}-3`, type: 'success', text: 'Cold Startup Time:           OxideCode: 42ms    | VS Code: 1250ms  (30x faster)' },
        { id: `${idPrefix}-4`, type: 'success', text: 'Idle RAM Consumption:        OxideCode: 22MB    | VS Code: 480MB   (22x leaner)' },
        { id: `${idPrefix}-5`, type: 'success', text: 'Frame Rate at Scroll:        OxideCode: 120 FPS | VS Code: 58 FPS  (Lossless GPU)' },
      ],
    };
  }

  if (trimmed === 'git status') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'output', text: `On branch ${currentBranch}` },
        { id: `${idPrefix}-2`, type: 'output', text: 'Changes not staged for commit:' },
        { id: `${idPrefix}-3`, type: 'error', text: '  modified:   src/main.rs' },
        { id: `${idPrefix}-4`, type: 'error', text: '  modified:   Cargo.toml' },
        { id: `${idPrefix}-5`, type: 'output', text: 'Untracked files:' },
        { id: `${idPrefix}-6`, type: 'error', text: '  benches/latency.rs' },
        { id: `${idPrefix}-7`, type: 'info', text: 'no changes added to commit (use "git add" to stage)' },
      ],
    };
  }

  if (trimmed === 'git log' || trimmed === 'git log --oneline') {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'info', text: 'e89a421 (HEAD -> main) perf: implement SIMD utf-8 line scanning' },
        { id: `${idPrefix}-2`, type: 'output', text: '9b24cd1 feat: add VS Code extension host Wasm sandbox' },
        { id: `${idPrefix}-3`, type: 'output', text: 'a11ff7c feat: integrate rust-analyzer LSP client' },
        { id: `${idPrefix}-4`, type: 'output', text: '3c0e12d init: initial Rust core repository scaffold' },
      ],
    };
  }

  if (trimmed.startsWith('ls')) {
    return {
      lines: [
        { id: `${idPrefix}-1`, type: 'output', text: 'Cargo.lock  Cargo.toml  README.md  benches/  src/  target/  tests/' },
      ],
    };
  }

  return {
    lines: [
      { id: `${idPrefix}-err`, type: 'error', text: `command not found: ${trimmed}. Type 'help' for commands.` },
    ],
  };
}
