#!/usr/bin/env python3
"""Capture sidebar regression scenarios in the native app using a temporary Git workspace.

Requires an active desktop session. Never reads or changes the user's app settings
or Git index. The PNGs are the app's own framebuffer, not desktop screenshots.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCENARIOS = ("search", "git", "search-empty", "search-light", "search-error", "extensions", "explorer", "debug", "settings")


def capture(binary, output, scenarios):
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="oxide-ui-smoke-") as temporary:
        work = Path(temporary)
        project = work / "OxideCode"
        (project / "src").mkdir(parents=True)

        def git(*arguments):
            return subprocess.run([
                "git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false",
                "-c", "user.name=UI Preview", "-c", "user.email=preview@localhost",
                *arguments,
            ], cwd=project, check=True, capture_output=True, text=True)

        for name in ("app.rs", "search.rs", "ui_sidebar.rs", "theme.rs"):
            shutil.copy2(ROOT / "src" / name, project / "src" / name)
        (project / "Cargo.toml").write_text('[package]\nname = "oxide-code"\nversion = "0.1.0"\nedition = "2021"\n')
        (project / "README.md").write_text("# OxideCode\nNative Rust editor.\n")
        git("init", "-b", "main")
        git("add", ".")
        git("commit", "-m", "Initial preview workspace")
        git("branch", "feature/sidebar-polish")
        (project / "README.md").write_text("# OxideCode\nSaved layout notes.\n")
        git("stash", "push", "-m", "Layout adjustments")
        for name in ("app.rs", "search.rs", "ui_sidebar.rs", "theme.rs"):
            with (project / "src" / name).open("a") as file:
                file.write("\n// Sidebar layout refinement.\n")
        git("add", "src/ui_sidebar.rs", "src/theme.rs")
        (project / "README.md").write_text("# OxideCode\nRefined workspace navigation.\n")
        (project / "src/a_very_long_unicode_世界_filename_for_clipping.json").write_text('{"preview": true}\n')

        for scenario in scenarios:
            config = work / ("settings-" + scenario)
            config.mkdir()
            (config / "config.json").write_text(json.dumps({
                "theme": "VS Code Light Modern" if scenario == "search-light" else "VS Code Dark Modern",
                "minimap_enabled": True,
                "check_on_save": False,
            }))
            destination = output / (scenario + ".png")
            destination.unlink(missing_ok=True)
            environment = {key: value for key, value in os.environ.items() if not key.startswith("OXIDE_")}
            environment.update({
                "OXIDE_CONFIG_DIR": str(config), "OXIDE_CAPTURE": str(destination),
                "OXIDE_VIEW": "search" if scenario.startswith("search") else scenario,
                "OXIDE_OPEN": "src/app.rs", "OXIDE_HIDE_PANEL": "1",
                "OXIDE_EXPAND": "src",
            })
            if scenario.startswith("search") and scenario != "search-empty":
                environment.update({"OXIDE_SEARCH": "run_search", "OXIDE_SEARCH_INCLUDE": "src/**/*.rs"})
            if scenario == "search-error":
                environment.update({"OXIDE_SEARCH": "[", "OXIDE_SEARCH_REGEX": "1"})
            subprocess.run([str(binary), str(project)], env=environment, check=True, timeout=25)
            with destination.open("rb") as file:
                header = file.read(24)
            assert header[:8] == b"\x89PNG\r\n\x1a\n", destination
            width, height = struct.unpack(">II", header[16:24])
            assert width >= 960 and height >= 620 and destination.stat().st_size > 10000, destination
            print(f"Captured {scenario}: {destination} ({width}×{height})", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/oxide-code")
    parser.add_argument("--output", type=Path, default=ROOT / "docs/ui-redesign/actual")
    parser.add_argument("--scenario", action="append", choices=SCENARIOS)
    args = parser.parse_args()
    capture(args.binary.resolve(), args.output.resolve(), args.scenario or SCENARIOS)
