#!/usr/bin/env python3
"""CLI example using harp-python bindings."""

import argparse
import sys

import harp


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="harp",
        description="Generate random names",
    )
    parser.add_argument(
        "-n", "--count",
        type=int,
        default=1,
        help="Number of names to generate",
    )
    parser.add_argument(
        "-v", "--version",
        action="store_true",
        help="Show version and exit",
    )

    args = parser.parse_args()

    if args.version:
        print(f"harp {harp.version()}")
        return 0

    for _ in range(args.count):
        print(harp.generate_name())

    return 0


if __name__ == "__main__":
    sys.exit(main())
