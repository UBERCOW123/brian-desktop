"""Extract CORE kind strings from vendor/core for contract parity tests."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DART = ROOT / "vendor/core/lib/core/data/core_models.dart"
OUT = ROOT / "contracts/fixtures/parity_manifest.json"


def extract_enum(name: str, source: str) -> list[str]:
    pattern = rf"enum {name} \{{(.*?)}};"
    match = re.search(pattern, source, re.DOTALL)
    if not match:
        raise SystemExit(f"enum {name} not found")
    body = match.group(1)
    values = re.findall(r"\w+\('([^']+)'\)", body)
    return values


def main() -> None:
    source = DART.read_text(encoding="utf-8")
    manifest = {
        "source": "vendor/core/lib/core/data/core_models.dart",
        "record_kinds": extract_enum("CoreRecordKind", source),
        "event_kinds": extract_enum("CoreEventKind", source),
        "link_kinds": extract_enum("CoreLinkKind", source),
        "outbox_statuses": extract_enum("SyncOutboxStatus", source),
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {OUT}")


if __name__ == "__main__":
    main()
