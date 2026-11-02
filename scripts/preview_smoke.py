#!/usr/bin/env python3
"""Generate tiny real preview fixtures and check them using OxideCode's native readers."""
import argparse
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import zipfile
import zlib


def fixtures(root):
    root.mkdir(parents=True, exist_ok=True)
    (root / 'project.json').write_text(json.dumps({
        'name': 'OxideCode', 'runtime': 'Native Rust', 'version': '0.1.0',
        'previews': ['Images & SVG', 'CSV & Excel', 'PDF pages', 'Word documents', 'Video posters'],
        'editor': {'theme': 'Dark Modern', 'fontSize': 14, 'formatOnSave': True},
        'workspace': {'files': 126, 'modified': 3, 'branch': 'main'},
    }, indent=2) + '\n')
    (root / 'workspace.csv').write_text(
        'Feature,Implementation,Status,Category\n'
        'Native workbench,Rust + egui,Available,Editor\n'
        'Images and SVG,Native preview,Available,Media\n'
        'CSV and Excel,Virtualized table,Available,Data\n'
        'PDF viewer,Poppler pages,Available,Documents\n'
        'JSON tools,"Tree, beautify, minify",Available,Data\n'
        'VS Code themes,Original theme files,Available,Extensions\n'
        'Code extensions,Extension host,Pending,Extensions\n')
    # A generated PNG uses real image pixels, with no imaging dependency.
    width, height = 960, 540
    rows = b''.join(b'\0' + bytes(c for x in range(width) for c in (
        int(20 + x * 35 / width), int(70 + y * 90 / height), int(140 + x * 75 / width), 255))
        for y in range(height))
    def chunk(name, data):
        return struct.pack('>I', len(data)) + name + data + struct.pack('>I', zlib.crc32(name + data))
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b'')
    (root / 'gradient.png').write_bytes(png)
    # Minimal Office Open XML packages are independently readable by Office.
    ns = 'http://schemas.openxmlformats.org/'
    with zipfile.ZipFile(root / 'workbook.xlsx', 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr('[Content_Types].xml', f'<Types xmlns="{ns}package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>')
        z.writestr('_rels/.rels', f'<Relationships xmlns="{ns}package/2006/relationships"><Relationship Id="rId1" Type="{ns}officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>')
        z.writestr('xl/workbook.xml', f'<workbook xmlns="{ns}spreadsheetml/2006/main" xmlns:r="{ns}officeDocument/2006/relationships"><sheets><sheet name="Preview Support" sheetId="1" r:id="rId1"/></sheets></workbook>')
        z.writestr('xl/_rels/workbook.xml.rels', f'<Relationships xmlns="{ns}package/2006/relationships"><Relationship Id="rId1" Type="{ns}officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>')
        data = [['Format', 'Reader', 'View'], ['PNG / JPEG / SVG', 'Rust image + resvg', 'Zoom / fit'], ['CSV / TSV', 'Rust csv', 'Filter rows'], ['XLSX / XLS / ODS', 'Rust calamine', 'Worksheets'], ['PDF', 'Poppler', 'Page navigation'], ['DOCX', 'Rust ZIP + XML', 'Selectable text']]
        from xml.sax.saxutils import escape
        sheet_rows = ''.join('<row r="%d">%s</row>' % (i, ''.join(f'<c r="{chr(65+j)}{i}" t="inlineStr"><is><t>{escape(cell)}</t></is></c>' for j, cell in enumerate(row))) for i, row in enumerate(data, 1))
        z.writestr('xl/worksheets/sheet1.xml', f'<worksheet xmlns="{ns}spreadsheetml/2006/main"><dimension ref="A1:C6"/><sheetData>{sheet_rows}</sheetData></worksheet>')
    with zipfile.ZipFile(root / 'document.docx', 'w', zipfile.ZIP_DEFLATED) as z:
        z.writestr('[Content_Types].xml', f'<Types xmlns="{ns}package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        z.writestr('_rels/.rels', f'<Relationships xmlns="{ns}package/2006/relationships"><Relationship Id="rId1" Type="{ns}officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
        paragraphs = ['OxideCode document preview', 'A native Rust workspace for code and project files.', 'Open images, spreadsheets, JSON and PDF pages in editor tabs.', 'Word text stays selectable. Open in System App preserves the original document layout.', 'Unicode and entities: 中文 & notes.']
        z.writestr('word/document.xml', f'<w:document xmlns:w="{ns}wordprocessingml/2006/main"><w:body>' + ''.join(f'<w:p><w:r><w:t>{escape(text)}</w:t></w:r></w:p>' for text in paragraphs) + '</w:body></w:document>')
    stream = b'BT /F1 28 Tf 60 700 Td (OxideCode PDF Preview) Tj /F1 14 Tf 0 -50 Td (Native editor. Real project files.) Tj 0 -32 Td (Page rendering with zoom and navigation.) Tj ET'
    objects = [b'<< /Type /Catalog /Pages 2 0 R >>', b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>', b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>', b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>', b'<< /Length %d >>\nstream\n' % len(stream) + stream + b'\nendstream']
    pdf = b'%PDF-1.4\n'; offsets = []
    for i, obj in enumerate(objects, 1):
        offsets.append(len(pdf)); pdf += f'{i} 0 obj\n'.encode() + obj + b'\nendobj\n'
    start = len(pdf)
    pdf += b'xref\n0 6\n0000000000 65535 f \n' + b''.join(f'{offset:010d} 00000 n \n'.encode() for offset in offsets) + f'trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n'.encode()
    (root / 'document.pdf').write_bytes(pdf)
    return ['gradient.png', 'workspace.csv', 'workbook.xlsx', 'document.docx']


def run(binary, root, external):
    if not binary.exists() and binary.with_suffix(".exe").exists():
        binary = binary.with_suffix(".exe")
    files = fixtures(root)
    if external and shutil.which('pdftoppm'):
        files.append('document.pdf')
    if external and shutil.which('ffmpeg'):
        subprocess.run(['ffmpeg', '-v', 'error', '-y', '-f', 'lavfi', '-i', 'testsrc2=size=640x360:rate=12', '-t', '1', '-pix_fmt', 'yuv420p', str(root / 'video.mp4')], check=True)
        files.append('video.mp4')
    subprocess.run([str(binary.resolve()), '--check-preview', *(str(root / file) for file in files)], check=True)
    print('Preview smoke checks passed.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/oxide-code'))
    parser.add_argument('--output', type=Path)
    parser.add_argument('--external', action='store_true', help='Also test installed Poppler/FFmpeg')
    args = parser.parse_args()
    if args.output:
        run(args.binary, args.output, args.external)
    else:
        with tempfile.TemporaryDirectory(prefix='oxide-preview-smoke-') as temp:
            run(args.binary, Path(temp), args.external)
