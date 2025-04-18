from datetime import datetime

import argparse
import subprocess
import time
import json
import os


def init_bench_database():
    subprocess.run(
        [
            "sqlite3",
            "bench.db",
            "\"VACUUM;\""
        ],
        capture_output=True,
        text=True
    )


def solve_zero_by(variant):
    subprocess.run(
        [
            "cargo",
            "run",
            "--",
            "build",
            "zero-by",
            f"--variant={variant}",
            "--mode=overwrite"
        ],
        capture_output=True,
        text=True
    )


def flamegraph(variant):
    subprocess.run(
        [
            "cargo",
            "flamegraph",
            "--root=-E",
            "--bin",
            "nova",
            "--",
            "build",
            "zero-by",
            f"--variant={variant}",
            "--mode=overwrite"
        ],
    )


# COMMAND LINE PARSING

parser = argparse.ArgumentParser(
    description="Benchmarks for 'nova'"
)

parser.add_argument(
    "--note",
    type=str,
    help="A note for identifying the benchmark run.",
    default="No note."
)

parser.add_argument(
    "--samples",
    type=int,
    help="Number of runs whose runtime to average.",
    default=3
)

parser.add_argument(
    "--profile",
    type=bool,
    help="Generate a flamegraph.",
    default=False
)

args = parser.parse_args()
print(f"Acknowledged.\n \
        \nnote={args.note} \
        \nsamples={args.samples} \
        \nprofile={args.profile}\n"
      )

# DECLARATIONS

samples = args.samples
variants = [
    "2-10000-1-2",
    "2-10000-3-5-7-11",
    "4-10000-1-2",
    "4-10000-3-5-7-11",
]

db = os.environ.get("DATABASE", "solutions.db")
os.environ["DATABASE"] = "bench.db"

# FLAMEGRAPH

if args.profile:
    flamegraph("2-100000-1-2")

# BENCHMARKS

measurements = {}
for v in variants:
    measurements[v] = []
    for _ in range(samples):
        start = time.time()

        solve_zero_by(v)

        elapsed = time.time() - start
        measurements[v].append(elapsed)

    m = measurements[v]
    print(f"zero_by [{v}]: {sum(m) / len(m)} seconds")

# LOGGING

os.environ["DATABASE"] = db

now = datetime.now()
stamp = now.strftime("%Y-%m-%d_%H-%M-%S")
with open(f"dev/bench/{stamp}.json", "w") as f:
    measurements["note"] = args.note
    json.dump(measurements, f)
