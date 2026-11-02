#!/usr/bin/env python3
"""Check registry dependencies with OSV; reject new or expired advisory reviews."""
import argparse
from datetime import date
import json
import random
from pathlib import Path
import time
import tomllib
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def request_json(payload):
    request = urllib.request.Request('https://api.osv.dev/v1/querybatch',
        data=json.dumps(payload).encode(), headers={'Content-Type': 'application/json'})
    for attempt in range(2):
        try:
            with urllib.request.urlopen(request, timeout=20) as response:
                return json.load(response)
        except (urllib.error.URLError, TimeoutError):
            if attempt:
                raise
            time.sleep(min(2.0, 0.5 * (2 ** attempt)) + random.uniform(0, 0.25))


def scan(lockfile):
    packages = [p for p in tomllib.loads(lockfile.read_text())['package']
                if p.get('source', '').startswith('registry+')]
    queries = [{'package': {'name': p['name'], 'ecosystem': 'crates.io'}, 'version': p['version']}
               for p in packages]
    findings = []
    for start in range(0, len(queries), 500):
        batch = queries[start:start + 500]
        results = request_json({'queries': batch})['results']
        if len(results) != len(batch):
            raise RuntimeError('OSV returned an incomplete result set')
        for query, result in zip(batch, results):
            ids = {v['id'] for v in result.get('vulns', [])}
            for _ in range(10):
                token = result.get('next_page_token')
                if not token:
                    break
                result = request_json({'queries': [{**query, 'page_token': token}]})['results'][0]
                ids.update(v['id'] for v in result.get('vulns', []))
            else:
                raise RuntimeError('OSV pagination exceeded its safety limit')
            if ids:
                findings.append({'name': query['package']['name'], 'version': query['version'],
                                 'advisories': sorted(ids)})
    return {'packages_queried': len(packages), 'findings': findings}


def violations(result, policy, today=None):
    today = today or date.today()
    reviews = {(p['name'], p['version'], p['advisory']): p for p in policy['reviews']}
    problems = []
    for package in result['findings']:
        for identifier in package['advisories']:
            key = (package['name'], package['version'], identifier)
            review = reviews.get(key)
            if not review or not review.get('reason'):
                problems.append(f'Unreviewed: {key[0]} {key[1]} {key[2]}')
            elif date.fromisoformat(review['review_by']) < today:
                problems.append(f'Review expired: {key[0]} {key[1]} {key[2]}')
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, help='Validate a saved OSV response without network access')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = json.loads(args.input.read_text()) if args.input else scan(ROOT / 'Cargo.lock')
    policy = json.loads((ROOT / 'security/advisory-reviews.json').read_text())
    problems = violations(result, policy)
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(f"Checked {result['packages_queried']} public registry packages; {len(result['findings'])} packages have advisory matches.")
    for problem in problems:
        print(problem)
    if problems:
        raise SystemExit(1)
    print('All current matches have explicit, time-limited reachability/maintenance reviews; this does not mean no advisories exist.')


if __name__ == '__main__':
    main()
