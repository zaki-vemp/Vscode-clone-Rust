import test from 'node:test';
import assert from 'node:assert/strict';
import { RopeBuffer } from './ropeBuffer.ts';
import { executeCommand } from './terminalEngine.ts';

test('RopeBuffer handles text insertion and line count', () => {
  const buffer = new RopeBuffer('fn main() {\n    println!("hello");\n}');
  assert.equal(buffer.getLineCount(), 3);
  assert.equal(buffer.getLine(1), 'fn main() {');
  assert.equal(buffer.getLine(2), '    println!("hello");');

  // Insert single line
  buffer.insert(1, 12, ' // entry');
  assert.equal(buffer.getLine(1), 'fn main() { // entry');

  // Insert multi-line
  buffer.insert(2, 5, 'let x = 42;\n    ');
  assert.equal(buffer.getLineCount(), 4);
  assert.equal(buffer.getLine(2), '    let x = 42;');
  assert.equal(buffer.getLine(3), '    println!("hello");');
});

test('RopeBuffer handles search query positions', () => {
  const buffer = new RopeBuffer('fn run() -> Result<(), Error> {\n    let status = run();\n}');
  const results = buffer.search('run');
  assert.equal(results.length, 2);
  assert.equal(results[0].line, 1);
  assert.equal(results[0].column, 4);
  assert.equal(results[1].line, 2);
  assert.equal(results[1].column, 18);
});

test('TerminalEngine handles cargo and rustc commands', () => {
  const rustcRes = executeCommand('rustc --version');
  assert.equal(rustcRes.lines.length, 1);
  assert.match(rustcRes.lines[0].text, /rustc 1.82.0/);

  const checkRes = executeCommand('cargo check');
  assert.equal(checkRes.lines.length, 4);
  assert.match(checkRes.lines[3].text, /Finished `dev` profile/);

  const testRes = executeCommand('cargo test');
  assert.equal(testRes.lines.some((l) => l.text.includes('5 passed')), true);

  const benchRes = executeCommand('bench');
  assert.equal(benchRes.lines.some((l) => l.text.includes('30x faster')), true);
});
