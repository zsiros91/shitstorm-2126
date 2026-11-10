# SHITSTORM 2126

Top-down post-apocalyptic pixel-art shooter built with [Bevy](https://bevy.org) 0.20.

## Running

```bash
cargo run --release
```

The game looks for assets in this order: `$BEVY_ASSET_ROOT/assets`, the project's
`assets/` folder (development builds), then an `assets/` folder next to the
executable. A packaged build therefore needs `assets/` beside the binary.

## Controls

| Input | Action |
| --- | --- |
| WASD / arrows | Move |
| Shift | Sprint (drains stamina) |
| Mouse | Aim |
| Left mouse | Fire |
| R | Reload |
| Esc | Pause |

## Building for Windows

### From Windows

Install Rust (MSVC toolchain) plus the Visual Studio C++ build tools, then:

```bash
cargo build --release
```

Ship `target/release/shitstorm-2126.exe` together with `assets/`.

### Cross-compiling from Linux

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt-get install gcc-mingw-w64-x86-64
cargo build --release --target x86_64-pc-windows-gnu
tools/package_windows.sh
```

The last command writes `dist/shitstorm-2126-windows-x86_64.zip`, which contains
the executable and `assets/`. Unpack it anywhere and run
`shitstorm-2126.exe`; release builds are windowed (no console).

The game defaults to the DX12 backend on Windows. Override it with
`WGPU_BACKEND=vulkan` (or another wgpu backend) if needed.

## Development

- `cargo test` runs the collision unit tests.
- F1 toggles the collider overlay, F2 spawns a ring of hostiles, F12 saves a
  screenshot.
- Environment flags used by the capture harness:
  `SHITSTORM_AUTOSTART`, `SHITSTORM_DEMO_INPUT`, `SHITSTORM_SCREENSHOT`,
  `SHITSTORM_SCREENSHOT_DELAY`.
- `tools/gen_assets.py` regenerates every sprite procedurally (Pillow).
