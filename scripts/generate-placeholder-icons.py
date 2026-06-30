"""Generate placeholder Tauri icon set until a brand asset is provided."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / "src-tauri" / "icons"
COLOR = (0x42, 0x85, 0xF4, 0xFF)  # CORE blue


def png_bytes(size: int) -> bytes:
    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    raw = b""
    row = bytes([0]) + bytes(COLOR[:3]) * size
    for _ in range(size):
        raw += row

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_png(path: Path, size: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png_bytes(size))


def write_ico(path: Path) -> None:
    # Minimal single 32x32 ICO embedding our PNG-like BMP payload.
    bmp_header = struct.pack("<IIIHHIIIIII", 40, 32, 64, 1, 32, 0, 32 * 64, 0, 0, 0, 0)
    pixels = b""
    for _ in range(32):
        pixels += b"\x00" + bytes(COLOR[:3]) * 32
    and_mask = b"\x00" * (32 * 4)
    image = bmp_header + pixels + and_mask
    ico = struct.pack("<HHH", 0, 1, 1)
    ico += struct.pack("<BBBBHHII", 32, 32, 0, 0, 1, 32, 22, 32 * 32 * 4)
    ico += image
    path.write_bytes(ico)


def main() -> None:
    write_png(ICONS / "32x32.png", 32)
    write_png(ICONS / "128x128.png", 128)
    write_png(ICONS / "128x128@2x.png", 256)
    write_ico(ICONS / "icon.ico")
    # icns is macOS-only; copy largest PNG as a stub (replace via `npm run tauri icon`)
    (ICONS / "icon.icns").write_bytes(png_bytes(256))
    print(f"Wrote placeholder icons to {ICONS}")


if __name__ == "__main__":
    main()
