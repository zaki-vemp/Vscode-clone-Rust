#!/usr/bin/env python3
"""Probe only a generated local file and our own loopback HTTP server."""
import argparse
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading


def probe(binary, expect_blocked=False):
    hits = []
    with tempfile.TemporaryDirectory(prefix='oxide-media-probe-') as temp:
        root = Path(temp)
        segment = root / 'segment.ts'
        subprocess.run(['ffmpeg', '-v', 'error', '-y', '-f', 'lavfi', '-i', 'color=c=blue:s=64x64:r=1', '-t', '1', '-c:v', 'mpeg2video', '-f', 'mpegts', str(segment)], check=True)
        payload = segment.read_bytes()
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                hits.append(self.path)
                self.send_response(200)
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            def log_message(self, *_):
                pass
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            file = root / 'untrusted.mp4'
            file.write_text('#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:1\n#EXT-X-MEDIA-SEQUENCE:0\n#EXTINF:1.0,\nhttp://127.0.0.1:%d/segment.ts\n#EXT-X-ENDLIST\n' % server.server_port)
            result = subprocess.run([str(binary.resolve()), '--check-preview', str(file)], text=True, capture_output=True, timeout=15)
            record = {'loopback_requests': len(hits), 'preview_exit': result.returncode,
                      'blocked': not hits, 'diagnostic': result.stderr[-1500:], 'environment': 'generated fixture + owned 127.0.0.1 listener'}
            print(json.dumps(record))
            if expect_blocked:
                assert not hits, 'A local media preview unexpectedly made an HTTP request'
                assert result.returncode != 0, 'Network-referencing media should be rejected'
            return record
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('target/release/oxide-code'))
    parser.add_argument('--expect-blocked', action='store_true')
    args = parser.parse_args()
    probe(args.binary, args.expect_blocked)
