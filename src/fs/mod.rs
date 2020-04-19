pub mod structure;
pub mod storage;

use structure::*;
use storage::Storage;

use std::fmt::Debug;
use serde::{Serialize};
use serde::de::DeserializeOwned;

use anyhow::{Result, Error};

#[derive(Debug)]
pub struct Fs {
 superblock: Superblock,
 data_bitmap: DataBitmap,
 inode_bitmap: InodeBitmap,
 storage: Storage,
}

impl Fs {
  pub fn new(filename: &str) -> Result<Self> {
    let mut storage = Storage::new(filename)?;
    fn read_or_new<T: Default + Serialize + DeserializeOwned>
                    (storage: &mut Storage, offset: usize, size: usize) -> Result<T> {
      storage.read(offset, size).map_err(Error::msg)
        .and_then(|bytes| bincode::deserialize(&bytes).map_err(Error::msg))
        .or_else(|_| {
          let default: T = Default::default();
          bincode::serialize(&default).map_err(Error::msg)
            .and_then(|bytes| storage.write(offset, &bytes).map_err(Error::msg))
            .map(|_| default)
        })
    }
    let sb = read_or_new::<Superblock>(&mut storage, 0, SUPERBLOCK_SIZE)?;
    let data_bitmap = read_or_new::<DataBitmap>(&mut storage, sb.data_bitmap, DATA_BITMAP_SIZE)?;
    let inode_bitmap = read_or_new::<InodeBitmap>(&mut storage, sb.inode_bitmap, INODES_BITMAP_SIZE)?;
    Ok(Fs {
      superblock: sb,
      data_bitmap: data_bitmap,
      inode_bitmap: inode_bitmap,
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