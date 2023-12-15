use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use rusqlite::Connection;

use crate::{errors::RpmdbError, DBI};

const SQLITE3_HEADER_MAGIC: &[u8] = b"SQLite format 3\x00";

pub struct SqliteDB {
    conn: rusqlite::Connection,
}

impl SqliteDB {
    fn check_sqlite<P: AsRef<Path>>(path: P) -> Result<(), RpmdbError> {
        let mut buf = [0; 16];
        let mut file = File::open(path)?;
        file.read_exact(&mut buf)?;

        if !buf.eq(SQLITE3_HEADER_MAGIC) {
            return Err(RpmdbError::InvalidSqliteFile);
        }

        Ok(())
    }

    pub fn open(path: PathBuf) -> Result<Self, RpmdbError> {
        Self::check_sqlite(&path)?;

        let conn = Connection::open(&path)?;

        Ok(Self { conn })
    }
}

impl DBI for SqliteDB {
    fn read(&mut self) -> Result<Vec<Vec<u8>>, RpmdbError> {
        let mut stmt = self.conn.prepare("SELECT blob FROM Packages")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut blobs = Vec::new();
        for row in rows {
            blobs.push(row?);
        }

        Ok(blobs)
    }
}

#[cfg(test)]
mod tests {
    use crate::DBI;

    use super::SqliteDB;

    const SQLITE_FILE: &str = "testdata/fedora35/rpmdb.sqlite";

    #[test]
    fn test_check_sqlite() {
        SqliteDB::check_sqlite(SQLITE_FILE).unwrap();
    }

    #[test]
    fn test_read() {
        let mut db = SqliteDB::open(SQLITE_FILE.parse().unwrap()).unwrap();
        let blobs = db.read().unwrap();
        assert!(!blobs.is_empty())
    }
}
