"""Command-line interface for PyLocate"""

import sys
import argparse
from pathlib import Path
from typing import Optional

from .db import Database


def format_size(size: int) -> str:
    """Format size in bytes to human-readable format"""
    for unit in ["B", "KB", "MB", "GB"]:
        if size < 1024.0:
            return f"{size:.1f} {unit}"
        size /= 1024.0
    return f"{size:.1f} TB"


def updatedb():
    """Update the database (like updatedb command)"""
    parser = argparse.ArgumentParser(
        description="Update PyLocate database", prog="updatedb-py"
    )
    parser.add_argument(
        "paths", nargs="*", help="Paths to index (default: home and system paths)"
    )
    parser.add_argument("--database", "-d", help="Database file path")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    args = parser.parse_args()

    db = Database(args.database)

    if args.verbose:
        print(f"Indexing filesystem...")
        if args.paths:
            print(f'Paths: {", ".join(args.paths)}')

    try:
        count = db.update(args.paths if args.paths else None)
        print(f"✓ Indexed {count:,} files")

        file_count, db_size = db.stats()
        print(f"Database: {db.db_path}")
        print(f"Size: {format_size(db_size)}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


def main():
    """Main CLI entry point (like locate command)"""
    parser = argparse.ArgumentParser(
        description="Find files by name using PyLocate database", prog="pylocate"
    )
    parser.add_argument(
        "pattern", nargs="?", help="Search pattern (supports * and ? wildcards)"
    )
    parser.add_argument("--database", "-d", help="Database file path")
    parser.add_argument(
        "--limit",
        "-l",
        type=int,
        default=1000,
        help="Limit number of results (default: 1000)",
    )
    parser.add_argument(
        "--count", "-c", action="store_true", help="Only show count of matches"
    )
    parser.add_argument(
        "--ignore-case", "-i", action="store_true", help="Ignore case distinctions"
    )
    parser.add_argument(
        "--basename", "-b", action="store_true", help="Match only the base name"
    )
    parser.add_argument(
        "--stats", "-s", action="store_true", help="Show database statistics"
    )
    parser.add_argument(
        "--update", "-u", action="store_true", help="Update database before searching"
    )

    args = parser.parse_args()

    db = Database(args.database)

    # Show stats
    if args.stats:
        if not db.exists():
            print("Database does not exist. Run 'updatedb-py' first.", file=sys.stderr)
            sys.exit(1)

        file_count, db_size = db.stats()
        print(f"Database: {db.db_path}")
        print(f"Files: {file_count:,}")
        print(f"Size: {format_size(db_size)}")
        return

    # Update database if requested
    if args.update:
        print("Updating database...", file=sys.stderr)
        try:
            count = db.update()
            print(f"✓ Indexed {count:,} files", file=sys.stderr)
        except Exception as e:
            print(f"Error updating database: {e}", file=sys.stderr)
            sys.exit(1)

    # Check if pattern provided
    if not args.pattern:
        parser.print_help()
        sys.exit(1)

    # Check if database exists
    if not db.exists():
        print("Database does not exist. Run 'updatedb-py' first.", file=sys.stderr)
        sys.exit(1)

    # Prepare pattern
    pattern = args.pattern
    if args.ignore_case:
        pattern = pattern.lower()

    if args.basename:
        pattern = f"*/{pattern}"

    # Search
    try:
        results = db.search(pattern, args.limit)

        # Filter by case if needed
        if args.ignore_case:
            results = [r for r in results if args.pattern.lower() in r.lower()]

        # Filter by basename if needed
        if args.basename:
            results = [r for r in results if args.pattern in Path(r).name]

        if args.count:
            print(len(results))
        else:
            for path in results:
                print(path)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
