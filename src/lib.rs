use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use jwalk::WalkDir;
use rusqlite::{Connection, params};
use std::path::Path;
use std::time::SystemTime;

/// Initialize the database with tables and triggers
fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    // Create main files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            inode INTEGER,
            mtime INTEGER,
            size INTEGER
        )",
        [],
    )?;

    // Create FTS5 virtual table
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
            path,
            content='files',
            content_rowid='id',
            tokenize = 'unicode61'
        )",
        [],
    )?;

    // Create trigger for INSERT
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
            INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
        END",
        [],
    )?;

    // Create trigger for DELETE
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
            INSERT INTO files_fts(files_fts, rowid, path)
            VALUES('delete', old.id, old.path);
        END",
        [],
    )?;

    // Create trigger for UPDATE
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
            INSERT INTO files_fts(files_fts, rowid, path)
            VALUES('delete', old.id, old.path);
            INSERT INTO files_fts(rowid, path) VALUES (new.id, new.path);
        END",
        [],
    )?;

    Ok(())
}

/// Index a directory and store results in SQLite
#[pyfunction]
fn index_directory(db_path: String, root_paths: Vec<String>) -> PyResult<usize> {
    let mut conn = Connection::open(&db_path)
        .map_err(|e| PyIOError::new_err(format!("Failed to open database: {}", e)))?;

    init_db(&conn)
        .map_err(|e| PyIOError::new_err(format!("Failed to initialize database: {}", e)))?;

    // Clear existing data
    conn.execute("DELETE FROM files", [])
        .map_err(|e| PyIOError::new_err(format!("Failed to clear database: {}", e)))?;

    let tx = conn.transaction()
        .map_err(|e| PyIOError::new_err(format!("Failed to start transaction: {}", e)))?;

    let mut count = 0usize;
    let mut stmt = tx.prepare(
        "INSERT INTO files (path, inode, mtime, size) VALUES (?1, ?2, ?3, ?4)"
    ).map_err(|e| PyIOError::new_err(format!("Failed to prepare statement: {}", e)))?;

    for root_path in root_paths {
        for entry in WalkDir::new(&root_path)
            .skip_hidden(false)
            .follow_links(false)
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let path_str = path.to_string_lossy().to_string();

                    if let Ok(metadata) = entry.metadata() {
                        let inode = get_inode(&metadata);
                        let mtime = metadata.modified()
                            .ok()
                            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);
                        let size = metadata.len() as i64;

                        if stmt.execute(params![path_str, inode, mtime, size]).is_ok() {
                            count += 1;
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }

    drop(stmt);
    tx.commit()
        .map_err(|e| PyIOError::new_err(format!("Failed to commit transaction: {}", e)))?;

    Ok(count)
}

#[cfg(unix)]
fn get_inode(metadata: &std::fs::Metadata) -> i64 {
    use std::os::unix::fs::MetadataExt;
    metadata.ino() as i64
}

#[cfg(not(unix))]
fn get_inode(_metadata: &std::fs::Metadata) -> i64 {
    0
}

/// Search for files matching a pattern
#[pyfunction]
fn search_files(db_path: String, pattern: String, limit: Option<usize>) -> PyResult<Vec<String>> {
    let conn = Connection::open(&db_path)
        .map_err(|e| PyIOError::new_err(format!("Failed to open database: {}", e)))?;

    let query = if pattern.contains('*') || pattern.contains('?') {
        // Use LIKE for glob patterns
        let like_pattern = pattern.replace('*', "%").replace('?', "_");
        format!("SELECT path FROM files WHERE path LIKE ? ESCAPE '\\' ORDER BY path LIMIT ?")
    } else {
        // Use FTS5 for full-text search
        format!("SELECT files.path FROM files_fts 
                 JOIN files ON files_fts.rowid = files.id 
                 WHERE files_fts MATCH ? 
                 ORDER BY rank LIMIT ?")
    };

    let limit_val = limit.unwrap_or(1000) as i64;
    let mut stmt = conn.prepare(&query)
        .map_err(|e| PyIOError::new_err(format!("Failed to prepare query: {}", e)))?;

    let search_pattern = if pattern.contains('*') || pattern.contains('?') {
        pattern.replace('*', "%").replace('?', "_")
    } else {
        format!("*{}*", pattern)
    };

    let results = stmt.query_map(params![search_pattern, limit_val], |row| {
        row.get::<_, String>(0)
    })
    .map_err(|e| PyIOError::new_err(format!("Failed to execute query: {}", e)))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(results)
}

/// Get database statistics
#[pyfunction]
fn get_stats(db_path: String) -> PyResult<(usize, i64)> {
    let conn = Connection::open(&db_path)
        .map_err(|e| PyIOError::new_err(format!("Failed to open database: {}", e)))?;

    let count: usize = conn.query_row(
        "SELECT COUNT(*) FROM files",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    let size: i64 = std::fs::metadata(&db_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    Ok((count, size))
}

#[pymodule]
fn pylocate_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(index_directory, m)?)?;
    m.add_function(wrap_pyfunction!(search_files, m)?)?;
    m.add_function(wrap_pyfunction!(get_stats, m)?)?;
    Ok(())
}
