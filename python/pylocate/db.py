"""Database management for PyLocate"""

import os
from pathlib import Path
from typing import List, Optional

try:
    from .pylocate_rust import index_directory, search_files, get_stats
except ImportError:
    from pylocate_rust import index_directory, search_files, get_stats


class Database:
    """Manage the PyLocate database"""

    def __init__(self, db_path: Optional[str] = None):
        """Initialize database manager

        Args:
            db_path: Path to database file. If None, uses default location.
        """
        if db_path is None:
            # Support both standard Linux and Termux
            if os.environ.get("TERMUX_VERSION"):
                base_dir = Path.home() / ".local" / "var" / "pylocate"
            else:
                base_dir = Path.home() / ".local" / "var" / "pylocate"

            base_dir.mkdir(parents=True, exist_ok=True)
            self.db_path = str(base_dir / "pylocate.db")
        else:
            self.db_path = db_path

    def update(self, paths: Optional[List[str]] = None) -> int:
        """Update the database by indexing filesystem

        Args:
            paths: List of paths to index. If None, indexes common locations.

        Returns:
            Number of files indexed
        """
        if paths is None:
            # Default paths to index
            if os.environ.get("TERMUX_VERSION"):
                # Termux-specific paths
                paths = [
                    str(Path.home()),
                    "/data/data/com.termux/files/usr",
                ]
            else:
                # Standard Linux paths
                paths = [
                    str(Path.home()),
                    "/usr",
                    "/opt",
                    "/var",
                ]

            # Filter to only existing paths
            paths = [p for p in paths if os.path.exists(p)]

        return index_directory(self.db_path, paths)

    def search(self, pattern: str, limit: Optional[int] = None) -> List[str]:
        """Search for files matching pattern

        Args:
            pattern: Search pattern (supports * and ? wildcards)
            limit: Maximum number of results

        Returns:
            List of matching file paths
        """
        return search_files(self.db_path, pattern, limit)

    def stats(self) -> tuple:
        """Get database statistics

        Returns:
            Tuple of (file_count, db_size_bytes)
        """
        return get_stats(self.db_path)

    def exists(self) -> bool:
        """Check if database exists"""
        return os.path.exists(self.db_path)
