#!/usr/bin/env python3
"""Verify a generated Debian package without installing it (Linux only)."""
import argparse
import hashlib
import io
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


def check(package):
    package = package.resolve()
    fields = subprocess.check_output(["dpkg-deb", "--field", str(package)], text=True)
    assert "Package: oxide-code\n" in fields, fields
    assert "Depends:" in fields and "libc6" in fields and "libxkbcommon0" in fields, fields
    with package.open("rb") as file:
        digest = hashlib.file_digest(file, "sha256").hexdigest()
    assert package.with_name(package.name + ".sha256").read_text() == f"{digest}  {package.name}\n"
    data = subprocess.check_output(["dpkg-deb", "--fsys-tarfile", str(package)])
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        members = {member.name.removeprefix("./"): member for member in archive.getmembers()}
        assert all(member.uid == member.gid == 0 for member in members.values()), "Non-root ownership"
        for name, mode in [
            ("usr/bin/oxide-code", 0o755),
            ("usr/share/applications/oxide-code.desktop", 0o644),
            ("usr/share/icons/hicolor/scalable/apps/oxide-code.svg", 0o644),
            ("usr/share/doc/oxide-code/copyright", 0o644),
            ("usr/share/doc/oxide-code/THIRD_PARTY_NOTICES.md", 0o644),
        ]:
            assert members[name].isfile() and members[name].mode == mode, name
    with tempfile.TemporaryDirectory(prefix="oxide-deb-check-") as directory:
        subprocess.run(["dpkg-deb", "--extract", str(package), directory], check=True)
        binary = Path(directory) / "usr/bin/oxide-code"
        version = subprocess.check_output([str(binary), "--version"], text=True).strip()
        assert version.startswith("OxideCode "), version
        if shutil.which("desktop-file-validate"):
            subprocess.run(["desktop-file-validate", str(Path(directory) / "usr/share/applications/oxide-code.desktop")], check=True)
    print(f"Verified {package.name}: metadata, checksum, ownership, permissions, launcher, and {version}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path)
    check(parser.parse_args().package)
