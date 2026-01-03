# PyLocate

A fast file locator tool implemented in Python with a Rust backend using `jwalk` for filesystem traversal and SQLite with FTS5 for searching.

## Features

- 🚀 Fast filesystem traversal using Rust's `jwalk`
- 🔍 Full-text search with SQLite FTS5
- 📱 Termux support
- 🎯 Glob pattern matching (* and ?)
- 💾 Efficient database storage

## Installation
use pip
pip install git+https://github.com/unforgivenii147/pylocate

### Prerequisites

- Python 3.8+
- Rust toolchain
- Maturin

### Build and Install

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop --release

# Or build wheel
maturin build --release
pip install target/wheels/pylocate-*.whl


# after instal (first run)

# Index default paths
updatedb-py

# Index specific paths
updatedb-py /path/to/index /another/path

# Verbose output
updatedb-py -v



## Building and Testing

```bash
# Install dependencies
pip install maturin

# Development build
maturin develop --release

# Create database
updatedb-py

# Search
pylocate test
pylocate "*.py"
pylocate --stats

# Build wheel for distribution
maturin build --release





# basic usage

# Simple search
pylocate myfile

# With wildcards
pylocate "*.py"
pylocate "test?.txt"

# Case-insensitive
pylocate -i MyFile

# Match basename only
pylocate -b config.json

# Limit results
pylocate -l 100 document

# Show count only
pylocate -c "*.pdf"

# Show statistics
pylocate --stats

# Update and search
pylocate -u myfile

