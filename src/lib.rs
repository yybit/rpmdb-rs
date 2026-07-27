//! # rpmdb
//!
//! A Rust library for reading RPM package databases, ported from
//! [go-rpmdb](https://github.com/knqyf263/go-rpmdb).
//!
//! The library auto-detects the on-disk format from the file path you provide,
//! so you can pass any supported database file directly to [`read_packages`].
//!
//! ## Supported formats
//!
//! | Format | Typical file name |
//! |--------|-------------------|
//! | Berkeley DB (BDB) | `Packages` |
//! | New DB (NDB) | `Packages.db` |
//! | SQLite3 | `rpmdb.sqlite` |
//!
//! ## Example
//!
//! ```no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let packages = rpmdb::read_packages("testdata/Packages".parse()?)?;
//!     for package in packages {
//!         println!("{} {:?}", package.name, package.provides);
//!     }
//!     Ok(())
//! }
//! ```

#[allow(dead_code)]
mod bdb;
mod entry;
mod errors;
mod ndb;
mod package;
#[allow(dead_code)]
mod rpmtags;

mod sqlite3;

pub use errors::RpmdbError;
pub use package::Package;

use ndb::Ndb;
use sqlite3::SqliteDB;
use std::path::PathBuf;

use bdb::Bdb;
use entry::Hdrblob;

/// Low-level trait for reading raw header blobs from an RPM database.
///
/// Each `Vec<u8>` element in the returned vector is a single serialised RPM
/// header blob. Implement this trait to add support for new on-disk formats.
pub trait DBI {
    /// Read all package header blobs from the database.
    ///
    /// # Errors
    ///
    /// Returns [`RpmdbError`] if the underlying database file cannot be read or
    /// if a structural error is encountered during parsing.
    fn read(&mut self) -> Result<Vec<Vec<u8>>, RpmdbError>;
}

fn open(path: PathBuf) -> Result<Box<dyn DBI>, RpmdbError> {
    match SqliteDB::open(path.clone()) {
        Ok(db) => {
            return Ok(Box::new(db));
        }
        Err(RpmdbError::InvalidSqliteFile) => {}
        Err(e) => {
            return Err(e);
        }
    }

    match Ndb::open(path.clone()) {
        Ok(db) => {
            return Ok(Box::new(db));
        }
        Err(RpmdbError::InvalidNdbFile) => {}
        Err(e) => {
            return Err(e);
        }
    }

    Ok(Box::new(Bdb::open(path)?))
}

/// Read all installed packages from an RPM database file.
///
/// The format of the database (BDB, NDB, or SQLite3) is detected automatically
/// from the file contents.
///
/// # Arguments
///
/// * `path` – Filesystem path to the RPM database file (e.g. `/var/lib/rpm/Packages`).
///
/// # Returns
///
/// A [`Vec`] of [`Package`] structs, one per installed package recorded in the database.
///
/// # Errors
///
/// Returns [`RpmdbError`] if:
/// - the file cannot be opened or read,
/// - the file does not match any supported RPM database format, or
/// - a header blob or package entry cannot be parsed.
///
/// # Example
///
/// ```no_run
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let packages = rpmdb::read_packages("/var/lib/rpm/Packages".parse()?)?;
///     for package in packages {
///         println!("{}-{}-{}.{}", package.name, package.version, package.release, package.arch);
///     }
///     Ok(())
/// }
/// ```
pub fn read_packages(path: PathBuf) -> Result<Vec<Package>, RpmdbError> {
    let mut db = open(path)?;

    let mut packages = Vec::new();
    let values = db.read()?;
    for value in values.clone() {
        let blob =
            Hdrblob::from_bytes(value.clone()).map_err(|e| RpmdbError::ParseBlob(e.to_string()))?;
        let mut entries = blob
            .import(value)
            .map_err(|e| RpmdbError::ParseBlob(e.to_string()))?;
        entries.sort_by_key(|e| e.info.offset);
        let pkg = Package::try_from(entries).map_err(|e| RpmdbError::ParseEntry(e.to_string()))?;
        packages.push(pkg);
    }

    Ok(packages)
}

#[cfg(test)]
mod tests {
    use crate::{open, read_packages};

    #[test]
    fn test_open() {
        open("testdata/centos7-python35/Packages".parse().unwrap()).unwrap();
        open("testdata/fedora35/rpmdb.sqlite".parse().unwrap()).unwrap();
        open("testdata/sle15-bci/Packages.db".parse().unwrap()).unwrap();
    }

    #[test]
    fn test_read_packages() {
        let pkgs1 = read_packages("testdata/centos7-python35/Packages".parse().unwrap()).unwrap();
        assert!(!pkgs1.is_empty());

        let pkgs2 = read_packages("testdata/fedora35/rpmdb.sqlite".parse().unwrap()).unwrap();
        assert!(!pkgs2.is_empty());

        let pkgs3 = read_packages("testdata/sle15-bci/Packages.db".parse().unwrap()).unwrap();
        assert!(!pkgs3.is_empty());
    }
}
