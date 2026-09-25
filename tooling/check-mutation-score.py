#!/usr/bin/env python3
"""Fail when a cargo-mutants report is below the committed baseline."""

import json
import sys
from pathlib import Path


def rust_score(path: Path) -> float:
    payload = json.loads(path.read_text(encoding="utf-8"))
    caught = int(payload["caught"])
    missed = int(payload["missed"])
    timeout = int(payload["timeout"])
    total = caught + missed + timeout
    if total == 0:
        raise SystemExit(f"no scored mutants in {path}")
    return 100.0 * (caught + timeout) / total


def main() -> None:
    if len(sys.argv) != 4 or sys.argv[1] != "rust":
        raise SystemExit("usage: check-mutation-score.py rust REPORT MINIMUM")
    report, minimum_text = sys.argv[2:]
    minimum = float(minimum_text)
    if minimum >= 100:
        raise SystemExit("minimum mutation score must be below 100")
    score = rust_score(Path(report))
    print(f"rust mutation score {score:.2f} (minimum {minimum:.2f})")
    if score + 1e-9 < minimum:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
