#[allow(dead_code)]
mod bdb;
mod entry;
mod errors;
mod ndb;
mod package;
#[allow(dead_code)]
mod rpmtags;

mod sqlite3;

use errors::RpmdbError;
use ndb::Ndb;
use sqlite3::SqliteDB;
use std::path::PathBuf;

use bdb::Bdb;
use entry::Hdrblob;
use package::Package;

pub trait DBI {
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
