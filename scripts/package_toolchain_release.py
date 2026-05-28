#!/usr/bin/env python3

from __future__ import annotations

import argparse
import tarfile
import zipfile
from pathlib import Path


def package_tar_gz(source: Path, output: Path, binary_name: str) -> None:
    with tarfile.open(output, "w:gz") as archive:
        archive.add(source, arcname=f"bin/{binary_name}")


def package_zip(source: Path, output: Path, binary_name: str) -> None:
    mode = source.stat().st_mode
    arcname = f"bin/{binary_name}"
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        info = zipfile.ZipInfo(arcname)
        info.external_attr = (mode & 0xFFFF) << 16
        info.create_system = 3
        info.compress_type = zipfile.ZIP_DEFLATED
        with source.open("rb") as src:
            archive.writestr(info, src.read())


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Package a toolchain binary into a release archive."
    )
    parser.add_argument("--source", required=True, help="Path to the built binary")
    parser.add_argument("--output", required=True, help="Path to the archive to create")
    parser.add_argument(
        "--binary-name",
        required=True,
        help="Name of the binary inside the archive, without any platform extension",
    )
    parser.add_argument(
        "--format",
        required=True,
        choices=("tar.gz", "zip"),
        help="Archive format to create",
    )
    args = parser.parse_args()

    source = Path(args.source)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)

    if not source.is_file():
        raise SystemExit(f"source binary does not exist: {source}")

    if args.format == "tar.gz":
        package_tar_gz(source, output, args.binary_name)
    else:
        package_zip(source, output, args.binary_name)


if __name__ == "__main__":
    main()
