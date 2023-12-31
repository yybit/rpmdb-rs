use anyhow::anyhow;

use super::entry::IndexEntry;
use super::rpmtags::*;

#[derive(Default, Debug)]
pub struct Package {
    pub epoch: i32,
    pub name: String,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub source_rpm: String,
    pub size: i32,
    pub license: String,
    pub vendor: String,
    pub modularitylabel: String,

    pub base_names: Vec<String>,
    pub dir_indexes: Vec<i32>,
    pub dir_names: Vec<String>,

    pub provides: Vec<String>,
    pub requires: Vec<String>,
}

impl TryFrom<Vec<IndexEntry>> for Package {
    type Error = anyhow::Error;

    fn try_from(entries: Vec<IndexEntry>) -> Result<Self, Self::Error> {
        let mut package: Package = Default::default();
        for entry in entries {
            match entry.info.tag as u32 {
                RPMTAG_DIRINDEXES => {
                    if entry.info._type != RPM_INT32_TYPE {
                        return Err(anyhow!("invalid tag dir indexes"));
                    }
                    package.dir_indexes = entry.read_i32_array()?;
                }
                RPMTAG_DIRNAMES => {
                    if entry.info._type != RPM_STRING_ARRAY_TYPE {
                        return Err(anyhow!("invalid tag dir names"));
                    }
                    package.dir_names = entry.read_string_array()?;
                }
                RPMTAG_BASENAMES => {
                    if entry.info._type != RPM_STRING_ARRAY_TYPE {
                        return Err(anyhow!("invalid tag base names"));
                    }
                    package.base_names = entry.read_string_array()?;
                }
                RPMTAG_MODULARITYLABEL => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag modularitylabel"));
                    }
                    package.modularitylabel = entry.read_string()?;
                }
                RPMTAG_NAME => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag name"));
                    }
                    package.name = entry.read_string()?;
                }
                RPMTAG_EPOCH => {
                    if entry.info._type != RPM_INT32_TYPE {
                        return Err(anyhow!("invalid tag epoch"));
                    }
                    package.epoch = entry.read_i32()?;
                }
                RPMTAG_VERSION => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag version"));
                    }
                    package.version = entry.read_string()?;
                }
                RPMTAG_RELEASE => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag release"));
                    }
                    package.release = entry.read_string()?;
                }
                RPMTAG_ARCH => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag arch"));
                    }
                    package.arch = entry.read_string()?;
                }
                RPMTAG_SOURCERPM => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag sourcerpm"));
                    }
                    package.source_rpm = entry.read_string()?;
                    if package.source_rpm == "(none)" {
                        package.source_rpm = "".to_string()
                    }
                }
                RPMTAG_LICENSE => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag license"));
                    }
                    package.license = entry.read_string()?;
                    if package.license == "(none)" {
                        package.license = "".to_string()
                    }
                }
                RPMTAG_VENDOR => {
                    if entry.info._type != RPM_STRING_TYPE {
                        return Err(anyhow!("invalid tag vendor"));
                    }
                    package.vendor = entry.read_string()?;
                    if package.vendor == "(none)" {
                        package.vendor = "".to_string()
                    }
                }
                RPMTAG_SIZE => {
                    if entry.info._type != RPM_INT32_TYPE {
                        return Err(anyhow!("invalid tag size"));
                    }
                    package.size = entry.read_i32()?;
                }
                RPMTAG_PROVIDENAME => {
                    if entry.info._type != RPM_STRING_ARRAY_TYPE {
                        return Err(anyhow!("invalid tag providename"));
                    }
                    package.provides = entry.read_string_array()?;
                }
                RPMTAG_REQUIRENAME => {
                    if entry.info._type != RPM_STRING_ARRAY_TYPE {
                        return Err(anyhow!("invalid tag requirename"));
                    }
                    package.requires = entry.read_string_array()?;
                }
                _ => {}
            }
        }

        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use crate::{bdb::Bdb, entry::Hdrblob, ndb::Ndb, sqlite3::SqliteDB, DBI};

    use super::Package;

    #[test]
    fn test_bdb() {
        let mut bdb = Bdb::open("testdata/centos7-python35/Packages".parse().unwrap()).unwrap();
        let values = bdb.read().unwrap();
        for value in values.clone() {
            let blob = Hdrblob::from_bytes(value.clone()).unwrap();
            let mut entries = blob.import(value).unwrap();
            entries.sort_by_key(|e| e.info.offset);
            let package = Package::try_from(entries).unwrap();
            println!("{} {:?}", package.name, package.provides);
        }
    }

    #[test]
    fn test_sqlite3() {
        let mut sqlite_db =
            SqliteDB::open("testdata/fedora35/rpmdb.sqlite".parse().unwrap()).unwrap();
        let values = sqlite_db.read().unwrap();
        for value in values.clone() {
            let blob = Hdrblob::from_bytes(value.clone()).unwrap();
            let mut entries = blob.import(value).unwrap();
            entries.sort_by_key(|e| e.info.offset);
            let package = Package::try_from(entries).unwrap();
            println!("{} {:?}", package.name, package.provides);
        }
    }

    #[test]
    fn test_ndb() {
        let mut ndb = Ndb::open("testdata/sle15-bci/Packages.db".parse().unwrap()).unwrap();
        let values = ndb.read().unwrap();
        for value in values.clone() {
            let blob = Hdrblob::from_bytes(value.clone()).unwrap();
            let mut entries = blob.import(value).unwrap();
            entries.sort_by_key(|e| e.info.offset);
            let package = Package::try_from(entries).unwrap();
            println!("{} {:?}", package.name, package.provides);
        }
    }
}
