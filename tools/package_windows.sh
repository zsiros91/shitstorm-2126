#!/usr/bin/env bash
# Bundle the Windows build into dist/shitstorm-2126-windows-x86_64.zip.
# Expects the release binary to already exist:
#   cargo build --release --target x86_64-pc-windows-gnu
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="x86_64-pc-windows-gnu"
binary="$root/target/$target/release/shitstorm-2126.exe"
dist="$root/dist"
out="$dist/shitstorm-2126-windows-x86_64.zip"
stage="$dist/stage"

if [[ ! -f "$binary" ]]; then
    echo "missing $binary" >&2
    echo "run: cargo build --release --target $target" >&2
    exit 1
fi

rm -rf "$stage" "$out"
mkdir -p "$stage"
cp "$binary" "$stage/"
cp -r "$root/assets" "$stage/assets"

# Bevy resolves assets next to the executable, so the zip layout is:
#   shitstorm-2126.exe
#   assets/...
python3 - "$stage" "$out" <<'PY'
import os, sys, zipfile

stage, out = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as archive:
    for folder, _, files in os.walk(stage):
        for name in files:
            path = os.path.join(folder, name)
            archive.write(path, os.path.relpath(path, stage))
print(f"wrote {out} ({os.path.getsize(out) / 1_048_576:.1f} MiB)")
PY

rm -rf "$stage"
