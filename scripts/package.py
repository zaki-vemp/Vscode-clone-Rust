#!/usr/bin/env python3
"""Build and package OxideCode for the current OS, using only Python's stdlib."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def copy_documentation(destination):
    for file in ["README.md", "LICENSE", "THIRD_PARTY_NOTICES.md"]:
        shutil.copy2(ROOT / file, destination / file)
    if (ROOT / "HARDENING_REPORT.md").is_file():
        shutil.copy2(ROOT / "HARDENING_REPORT.md", destination / "HARDENING_REPORT.md")
    shutil.copytree(ROOT / "docs", destination / "docs")
    licenses = destination / "licenses"
    licenses.mkdir()
    for source_file, target in [
        ("src/assets/codicon-LICENSE.txt", "codicons.txt"),
        ("src/assets/vscode-themes/LICENSE.txt", "vscode-themes.txt"),
        ("vendor/egui_term/LICENSE", "egui-term.txt"),
        ("vendor/egui_term/UPSTREAM.md", "egui-term-UPSTREAM.md"),
        ("src/assets/vscode-themes/SOURCE.md", "vscode-themes-SOURCE.md"),
    ]:
        shutil.copy2(ROOT / source_file, licenses / target)


def checksum(archive):
    with archive.open("rb") as archive_file:
        digest = hashlib.file_digest(archive_file, "sha256").hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
    print(f"Package: {archive}")
    print(f"SHA-256: {digest}")
    return archive


def package_deb(source, project, dist, maintainer):
    architecture = subprocess.check_output(["dpkg", "--print-architecture"], text=True).strip()
    with source.open("rb") as binary:
        header = binary.read(20)
    # Dependency discovery uses native system libraries, so reject cross builds.
    elf_targets = {"amd64": (2, 1, 62), "arm64": (2, 1, 183), "i386": (1, 1, 3)}
    target = (header[4], header[5], int.from_bytes(header[18:20], "little")) if len(header) == 20 else None
    if header[:4] != b"\x7fELF" or architecture not in elf_targets or target != elf_targets[architecture]:
        raise SystemExit(f"Debian packaging needs a native amd64, arm64, or i386 ELF binary (host: {architecture})")
    version = f"{project['version']}-1"
    archive = dist / f"oxide-code_{version}_{architecture}.deb"
    with tempfile.TemporaryDirectory(prefix="oxide-deb-") as temporary:
        work = Path(temporary)
        stage = work / "debian/oxide-code"
        for source_file, relative in [
            (source, "usr/bin/oxide-code"),
            (ROOT / "packaging/oxide-code.desktop", "usr/share/applications/oxide-code.desktop"),
            (ROOT / "packaging/oxide-code.svg", "usr/share/icons/hicolor/scalable/apps/oxide-code.svg"),
        ]:
            destination = stage / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source_file, destination)
        documentation = stage / "usr/share/doc/oxide-code"
        documentation.mkdir(parents=True)
        copy_documentation(documentation)
        shutil.copy2(ROOT / "LICENSE", documentation / "copyright")
        for path in stage.rglob("*"):
            path.chmod(0o755 if path.is_dir() else 0o644)
        stage.chmod(0o755)
        executable = stage / "usr/bin/oxide-code"
        executable.chmod(0o755)
        installed_size = sum((path.stat().st_size + 1023) // 1024 if path.is_file() else 1
                             for path in stage.rglob("*"))
        control = stage / "DEBIAN"
        control.mkdir(mode=0o755)
        control.chmod(0o755)
        (work / "debian/control").write_text(
            "Source: oxide-code\nSection: editors\nPriority: optional\n"
            f"Maintainer: {maintainer}\n\nPackage: oxide-code\nArchitecture: any\n"
            "Description: Native Rust code editor\n", encoding="utf-8")
        dependencies = subprocess.check_output(
            ["dpkg-shlibdeps", "-O", f"-e{executable.relative_to(work)}"], cwd=work, text=True)
        linked = next(line.removeprefix("shlibs:Depends=") for line in dependencies.splitlines()
                      if line.startswith("shlibs:Depends="))
        # The GUI loads these libraries with dlopen; ELF dependency scanning misses them.
        runtime = (
            "libegl1, libgl1, libx11-6, libx11-xcb1, libxcb1, libxcursor1, libxi6, "
            "libxkbcommon0, libxkbcommon-x11-0, libwayland-client0, "
            "libwayland-cursor0, libwayland-egl1, xdg-utils")
        (control / "control").write_text(
            f"Package: oxide-code\nVersion: {version}\nArchitecture: {architecture}\n"
            f"Section: editors\nPriority: optional\nMaintainer: {maintainer}\n"
            f"Installed-Size: {installed_size}\nDepends: {linked}, {runtime}\n"
            "Recommends: git\nSuggests: ffmpeg, poppler-utils\n"
            "Description: Native Rust desktop code editor\n"
            " A VS Code inspired workbench with native text editing, Git tools,\n"
            " terminals, themes, and file previews.\n", encoding="utf-8")
        (control / "control").chmod(0o644)
        subprocess.run(["dpkg-deb", "--root-owner-group", "--build", str(stage), str(archive)], check=True)
    return checksum(archive)


def package(no_build=False, package_format="portable", maintainer="OxideCode contributors <maintainer@localhost>"):
    system = platform.system()
    if package_format == "deb":
        if system != "Linux":
            raise SystemExit("Debian packages must be built on Debian/Ubuntu Linux")
        if not maintainer.strip() or any(ord(character) < 32 or ord(character) == 127 for character in maintainer):
            raise SystemExit("Maintainer must be a nonempty single line (Name <email>)")
        for tool in ("dpkg", "dpkg-deb", "dpkg-shlibdeps"):
            if not shutil.which(tool):
                raise SystemExit(f"Missing {tool}; install packaging tools with: sudo apt install dpkg-dev")
    if not no_build:
        subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version=1", "--locked"], cwd=ROOT))
    project = next(p for p in metadata["packages"] if p["name"] == "oxide-code")
    os_name = {"Linux": "linux", "Darwin": "macos", "Windows": "windows"}[system]
    arch = {"AMD64": "x86_64", "aarch64": "arm64"}.get(platform.machine(), platform.machine())
    name = f"oxide-code-{project['version']}-{os_name}-{arch}"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    binary = "oxide-code.exe" if system == "Windows" else "oxide-code"
    source = Path(metadata["target_directory"]) / "release" / binary
    if not source.is_file():
        raise SystemExit(f"Release binary missing: {source}; run without --no-build")
    if package_format == "deb":
        return package_deb(source, project, dist, maintainer)
    stage = dist / name
    # Delete only this script's own staging directory, never arbitrary paths.
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    if system == "Darwin":
        contents = stage / "OxideCode.app" / "Contents"
        executable = contents / "MacOS" / binary
        executable.parent.mkdir(parents=True)
        with (contents / "Info.plist").open("wb") as f:
            plistlib.dump({
                "CFBundleName": "OxideCode", "CFBundleDisplayName": "OxideCode",
                "CFBundleExecutable": binary, "CFBundleIdentifier": "dev.oxidecode.editor",
                "CFBundleVersion": project["version"], "CFBundleShortVersionString": project["version"],
                "CFBundlePackageType": "APPL", "NSHighResolutionCapable": True,
            }, f)
    else:
        executable = stage / binary
    shutil.copy2(source, executable)
    executable.chmod(0o755)
    copy_documentation(stage)
    if system == "Linux":
        shutil.copy2(ROOT / "packaging/oxide-code.desktop", stage)
        shutil.copy2(ROOT / "packaging/oxide-code.svg", stage)
    if system == "Windows":
        archive = dist / (name + ".zip")
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as f:
            for path in sorted(stage.rglob("*")):
                f.write(path, path.relative_to(dist))
    else:
        archive = dist / (name + ".tar.gz")
        with tarfile.open(archive, "w:gz") as f:
            f.add(stage, arcname=name)
    print(f"Binary:  {executable}")
    return checksum(archive)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true", help="Package the existing release binary")
    parser.add_argument("--format", choices=("portable", "deb"), default="portable", help="Package format (default: portable)")
    parser.add_argument("--maintainer", default="OxideCode contributors <maintainer@localhost>", help="Debian maintainer: Name <email>")
    args = parser.parse_args()
    package(args.no_build, args.format, args.maintainer)
