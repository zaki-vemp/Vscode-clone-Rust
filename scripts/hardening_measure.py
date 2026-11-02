#!/usr/bin/env python3
"""Reproduce the optimized scanner and repeated output-retention measurements."""
import argparse
import json
import platform
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
result = subprocess.run(['cargo', 'test', '--release', '--locked', 'hardening_measure', '--',
                         '--ignored', '--nocapture', '--test-threads=1'],
                        text=True, capture_output=True, check=True)
metrics = [json.loads(line.split('HARDENING_METRIC ', 1)[1])
           for line in result.stdout.splitlines() if 'HARDENING_METRIC ' in line]
if len(metrics) != 8:
    raise SystemExit('Incomplete benchmark output')
record = {'method': 'release ignored tests; 5 samples per scanner size, sequential; Rust drops instead of GC',
          'machine': platform.platform(),
          'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
          'metrics': metrics}
args.output.write_text(json.dumps(record, indent=2) + '\n')
print(args.output)
