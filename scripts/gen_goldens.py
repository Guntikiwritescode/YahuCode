#!/usr/bin/env python3
"""Generate golden fixtures from the behavioural oracle (oracle/yahucode_v1.py).

For each example this writes either:
  · example_NN.emit  — the exact emit() text (runnable examples), or
  · example_NN.diag  — the exact check() diagnostics, one per line (compile-only).

It also writes the example source to the examples/ directory. The goldens are the
*preserved behaviour* of the spike (handoff §5.4): once the ported Rust interpreter
reproduces them, the spike code is deleted (PR7) — behaviour is preserved via these
fixtures, not via the Python.

Usage:  python3 scripts/gen_goldens.py [golden_out_dir] [examples_out_dir]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "oracle"))
import yahucode_v1 as y  # noqa: E402

GOLDEN_OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "..", "tests", "golden")
EXAMPLES_OUT = sys.argv[2] if len(sys.argv) > 2 else os.path.join(HERE, "..", "examples")

# Stable slugs for the nine oracle examples (index → filename stem).
SLUGS = [
    "compiler_refuses",
    "protective_edge",
    "rising_lion",
    "iron_wall",
    "silent_shield",
    "guardian_of_the_walls",
    "eternal_shield",
    "swords_of_iron",
    "eternal_vigilance",
]


def main() -> None:
    os.makedirs(GOLDEN_OUT, exist_ok=True)
    os.makedirs(EXAMPLES_OUT, exist_ok=True)
    for i, (_title, src) in enumerate(y.EXAMPLES, start=1):
        stem = f"{i:02d}_{SLUGS[i - 1]}"
        with open(os.path.join(EXAMPLES_OUT, stem + ".yahu"), "w") as f:
            f.write(src.strip() + "\n")

        prog = y.parse(src)
        diags = y.check(prog)
        base = f"example_{i:02d}"
        if diags:
            with open(os.path.join(GOLDEN_OUT, base + ".diag"), "w") as f:
                f.write("\n".join(diags) + "\n")
            print(f"{base}.diag  ({len(diags)} diagnostics)")
        else:
            st = y.run(prog)
            with open(os.path.join(GOLDEN_OUT, base + ".emit"), "w") as f:
                f.write(y.emit(st) + "\n")
            print(f"{base}.emit  (discrepancies={st.discrepancies}, core={st.core})")


if __name__ == "__main__":
    main()
