pub mod structure;
pub mod storage;

use structure::*;
use storage::Storage;

use std::mem::size_of;
use std::fmt::Debug;

use anyhow::{Result, Error};

#[derive(Debug)]
pub struct Fs {
 superblock: Superblock,
 storage: Storage,
}

impl Fs {
  pub fn new(filename: &str) -> Result<Self> {
    let mut storage = Storage::new(filename)?;
    let sb = storage.read(0, SUPERBLOCK_SIZE).map_err(Error::msg)
      .and_then(|bytes| bincode::deserialize(&bytes).map_err(Error::msg))
      .unwrap_or({
        let default_sb: Superblock = Default::default();
        let sb_bytes = bincode::serialize(&default_sb)?;
        storage.write(0, &sb_bytes)?;
        default_sb
      });
    Ok(Fs {
      superblock: sb,
      storage: storage
    })
  }

  pub fn write_file(filename: &str, content: &Vec<u8>) -> Result<()> {
    Ok(())
  }

  pub fn read_file(filename: &str) -> Result<Vec<u8>> {
    Ok(vec![])
  }

  pub fn list_all(dirname: &str) -> Result<Vec<&Inode>> {
    Ok(vec![])
  }

} 