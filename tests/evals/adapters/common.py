from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any


class AdapterFailure(RuntimeError):
    pass


def fail(label: str, message: str) -> None:
    raise AdapterFailure(f"{label}: {message}")


def pass_case(label: str) -> None:
    print(f"  {label}: PASS")


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")


def envelope(
    *,
    run_id: str,
    phase: str,
    recipient: str,
    prompt: str,
    visible: list[dict[str, Any]],
    withheld: list[dict[str, Any]],
    redactions: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    return {
        "schema_version": "foundry.prompt-envelope.v1",
        "run_id": run_id,
        "phase": phase,
        "recipient": recipient,
        "prompt": prompt,
        "visible_context": visible,
        "withheld_context": withheld,
        "redactions": redactions or [],
    }


def validate_with_barrier(path: Path, barrier_validator: Path, label: str) -> None:
    proc = subprocess.run([str(barrier_validator), str(path)], text=True, capture_output=True)
    if proc.stdout:
        for line in proc.stdout.rstrip().splitlines():
            print(f"    {line}")
    if proc.stderr:
        print(proc.stderr, end="")
    if proc.returncode != 0:
        fail(label, f"barrier validator exited {proc.returncode} for {path}")


def require_columns(suite: str, cases: list[dict[str, str]], columns: list[str]) -> None:
    for index, case in enumerate(cases, start=1):
        missing = [column for column in columns if column not in case]
        if missing:
            fail(suite, f"case {index} missing columns: {', '.join(missing)}")
