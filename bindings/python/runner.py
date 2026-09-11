#!/usr/bin/env python3
"""Conformance test runner CLI for markstone Python bindings."""

import argparse
import sys
import markstone

def main():
    parser = argparse.ArgumentParser(description="markstone Python conformance runner")
    parser.add_argument("--mode", required=True, choices=["generic-html", "generic-ast", "actos-html", "actos-ast"])
    parser.add_argument("input_path", nargs="?", default=None, help="Path to input markdown file (or stdin if omitted)")

    args = parser.parse_args()

    if args.input_path and args.input_path != "-":
        with open(args.input_path, "r", encoding="utf-8") as f:
            content = f.read()
    else:
        content = sys.stdin.read()

    if args.mode == "generic-html":
        output = markstone.to_html(content)
    elif args.mode == "generic-ast":
        output = markstone.to_ast(content)
    elif args.mode == "actos-html":
        output = markstone.actos.to_html(content)
    elif args.mode == "actos-ast":
        output = markstone.actos.to_ast(content)
    else:
        sys.stderr.write(f"Unknown mode: {args.mode}\n")
        sys.exit(1)

    sys.stdout.buffer.write(output.encode("utf-8"))
    sys.stdout.buffer.flush()

if __name__ == "__main__":
    main()
