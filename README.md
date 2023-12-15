[![crates.io](https://img.shields.io/crates/v/rpmdb.svg)](https://crates.io/crates/rpmdb)
[![docs.rs](https://docs.rs/rpmdb/badge.svg)](https://docs.rs/rpmdb)

## rpmdb-rs

Rust implementation of rpmdb that ported from [go-rpmdb](https://github.com/knqyf263/go-rpmdb), currently only supports reading package list

Available rpmdb format:
- bdb
- ndb
- sqlite3

#### Example

```
let packages = rpmdb::read_packages("testdata/Packages".parse()?)?;
for package in packages {
    println!("{} {:?}", package.name, package.provides);
}
```