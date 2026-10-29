/**
 * RopeBuffer: High-performance text buffer model inspired by Rust's `ropey` crate.
 * Emulates sub-millisecond chunked text operations with line/column indices.
 */

export interface LineColumn {
  line: number; // 1-indexed
  column: number; // 1-indexed
}

export class RopeBuffer {
  private chunks: string[];

  constructor(initialText: string = '') {
    this.chunks = initialText.length === 0 ? [''] : initialText.split('\n');
  }

  public getText(): string {
    return this.chunks.join('\n');
  }

  public getLineCount(): number {
    return this.chunks.length;
  }

  public getLine(lineNumber: number): string {
    const idx = lineNumber - 1;
    if (idx < 0 || idx >= this.chunks.length) {
      return '';
    }
    return this.chunks[idx];
  }

  public insert(line: number, column: number, textToInsert: string): void {
    const lineIdx = Math.max(0, Math.min(line - 1, this.chunks.length - 1));
    const currentLine = this.chunks[lineIdx] || '';
    const colIdx = Math.max(0, Math.min(column - 1, currentLine.length));

    if (!textToInsert.includes('\n')) {
      // Fast single-line insert
      this.chunks[lineIdx] =
        currentLine.slice(0, colIdx) + textToInsert + currentLine.slice(colIdx);
    } else {
      // Multi-line insert
      const insertLines = textToInsert.split('\n');
      const head = currentLine.slice(0, colIdx) + insertLines[0];
      const tail = insertLines[insertLines.length - 1] + currentLine.slice(colIdx);
      const middle = insertLines.slice(1, -1);

      this.chunks.splice(lineIdx, 1, head, ...middle, tail);
    }
  }

  public delete(line: number, column: number, count: number): void {
    const lineIdx = Math.max(0, Math.min(line - 1, this.chunks.length - 1));
    const currentLine = this.chunks[lineIdx] || '';
    const colIdx = Math.max(0, Math.min(column - 1, currentLine.length));

    if (count <= 0) return;

    if (colIdx + count <= currentLine.length) {
      // Simple inline deletion
      this.chunks[lineIdx] =
        currentLine.slice(0, colIdx) + currentLine.slice(colIdx + count);
    } else {
      // Deletion spans newline
      if (lineIdx + 1 < this.chunks.length) {
        const nextLine = this.chunks[lineIdx + 1];
        const remaining = count - (currentLine.length - colIdx + 1); // 1 for newline
        this.chunks[lineIdx] = currentLine.slice(0, colIdx) + nextLine.slice(Math.max(0, remaining));
        this.chunks.splice(lineIdx + 1, 1);
      }
    }
  }

  public replaceText(newText: string): void {
    this.chunks = newText.split('\n');
  }

  public getCharCount(): number {
    let sum = 0;
    for (const chunk of this.chunks) {
      sum += chunk.length;
    }
    // Account for newlines
    return sum + Math.max(0, this.chunks.length - 1);
  }

  public search(query: string, caseSensitive: boolean = false): LineColumn[] {
    if (!query) return [];
    const results: LineColumn[] = [];
    const q = caseSensitive ? query : query.toLowerCase();

    for (let l = 0; l < this.chunks.length; l++) {
      const lineText = caseSensitive ? this.chunks[l] : this.chunks[l].toLowerCase();
      let startIdx = 0;
      while ((startIdx = lineText.indexOf(q, startIdx)) !== -1) {
        results.push({ line: l + 1, column: startIdx + 1 });
        startIdx += q.length;
      }
    }
    return results;
  }
}
