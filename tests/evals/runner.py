#!/usr/bin/env python3
"""Generic deterministic workflow eval runner for Foundry.

The runner intentionally supports a small Gherkin subset: feature files with an
Examples table. Suite-specific adapters turn rows into mocked workflow artifacts
and assertions.
"""
from __future__ import annotations

import argparse
import importlib
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class FeatureCases:
    suite: str
    path: Path
    cases: list[dict[str, str]]


class EvalFailure(RuntimeError):
    """Raised for user-facing eval failures."""


def split_table_row(line: str) -> list[str]:
    stripped = line.strip()
    if not (stripped.startswith("|") and stripped.endswith("|")):
        raise EvalFailure(f"not a Gherkin table row: {line!r}")
    return [cell.strip() for cell in stripped[1:-1].split("|")]


def parse_feature(path: Path) -> FeatureCases:
    rows: list[list[str]] = []
    in_examples = False
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped == "Examples:":
            in_examples = True
            continue
        if not in_examples:
            continue
        if not stripped:
            if rows:
                break
            continue
        if stripped.startswith("|"):
            rows.append(split_table_row(line))
        elif rows:
            break

    if len(rows) < 2:
        raise EvalFailure(f"{path}: feature must contain an Examples table with at least one row")

    header = rows[0]
    if any(not name for name in header):
        raise EvalFailure(f"{path}: Examples header contains an empty column name")

    cases: list[dict[str, str]] = []
    for row_number, row in enumerate(rows[1:], start=1):
        if len(row) != len(header):
            raise EvalFailure(
                f"{path}: Examples row {row_number} has {len(row)} cells, expected {len(header)}"
            )
        cases.append(dict(zip(header, row)))

    return FeatureCases(suite=path.stem, path=path, cases=cases)


def discover_features(root: Path, suite: str | None) -> list[Path]:
    features_dir = root / "tests" / "evals" / "features"
    if suite:
        path = features_dir / f"{suite}.feature"
        if not path.exists():
            raise EvalFailure(f"unknown suite {suite!r}; expected {path}")
        return [path]
    return sorted(features_dir.glob("*.feature"))


def adapter_for(suite: str):
    module_name = f"adapters.{suite.replace('-', '_')}"
    try:
        return importlib.import_module(module_name)
    except ModuleNotFoundError as exc:
        raise EvalFailure(f"{suite}: missing adapter module {module_name}") from exc


def run_suite(root: Path, feature: FeatureCases, keep_artifacts: bool) -> bool:
    adapter = adapter_for(feature.suite)
    barrier_validator = root / "tests" / "validate-barrier-envelopes.sh"
    print(f"== {feature.suite} ({feature.path.relative_to(root)}) ==")

    if keep_artifacts:
        base = root / "runs" / "evals" / feature.suite
        if base.exists():
            import shutil

            shutil.rmtree(base)
        base.mkdir(parents=True)
        adapter.run(root=root, feature_path=feature.path, cases=feature.cases, work_dir=base, barrier_validator=barrier_validator)
        print(f"{feature.suite}: PASS ({len(feature.cases)} cases, artifacts: {base.relative_to(root)})")
        return True

    with tempfile.TemporaryDirectory(prefix=f"foundry-{feature.suite}-evals-") as tmp:
        adapter.run(
            root=root,
            feature_path=feature.path,
            cases=feature.cases,
            work_dir=Path(tmp),
            barrier_validator=barrier_validator,
        )
    print(f"{feature.suite}: PASS ({len(feature.cases)} cases)")
    return True


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run deterministic Foundry workflow eval suites")
    parser.add_argument("features", nargs="*", type=Path, help="Specific .feature files to run")
    parser.add_argument("--suite", help="Run one suite from tests/evals/features/<suite>.feature")
    parser.add_argument("--keep-artifacts", action="store_true", help="Write artifacts under runs/evals/<suite>")
    args = parser.parse_args(argv)

    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root / "tests" / "evals"))

    try:
        feature_paths = args.features if args.features else discover_features(root, args.suite)
        feature_paths = [path.resolve() for path in feature_paths]
        if not feature_paths:
            raise EvalFailure("no eval feature files found")

        ok = True
        for path in feature_paths:
            feature = parse_feature(path)
            if args.suite:
                # Explicit feature paths may be compatibility fixtures whose basename differs
                # from the generic suite name, e.g. tests/fixtures/arbiter-routing-evals.feature.
                feature = FeatureCases(suite=args.suite, path=feature.path, cases=feature.cases)
            ok = run_suite(root, feature, args.keep_artifacts) and ok
        print(f"Foundry workflow evals: PASS ({len(feature_paths)} suite(s))")
        return 0 if ok else 1
    except EvalFailure as exc:
        print(f"Foundry workflow evals: FAIL — {exc}", file=sys.stderr)
        return 1
    except RuntimeError as exc:
        print(f"Foundry workflow evals: FAIL — {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
