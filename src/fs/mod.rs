pub mod structure;
pub mod storage;

use structure::*;
use storage::Storage;

use std::fmt;
use std::fmt::{Display, Debug};
use serde::{Serialize};
use serde::de::DeserializeOwned;

use anyhow::{anyhow, Result, Error};

#[derive(Debug)]
pub struct Fs {
 superblock: Superblock,
 data_bitmap: DataBitmap,
 inode_bitmap: InodeBitmap,
 storage: Storage,
 cur_dir: String,
}

impl Fs {

  fn write_inode(&mut self, inode: &Inode) -> Result<usize> {
    let ind = self.inode_bitmap.find_free().ok_or(anyhow!("Could not locate free inode"))?;
    let offset = self.superblock.inode_table + ind * self.superblock.inode_size;
    let bytes = bincode::serialize(inode)?;
    self.storage.write(offset, &bytes)?;
    self.inode_bitmap.set(ind, true)?;
    Ok(ind)
  }

  fn read_data(&self, inode_ind: usize) -> Result<Vec<u8>> {
    let inode_from = self.superblock.inode_table + inode_ind * INODE_SIZE;
    let inode_bytes = self.storage.read(inode_from, INODE_SIZE)?;
    let inode: Inode = bincode::deserialize(&inode_bytes)?;
    let data_indices = &inode.direct[0..(inode.size as f64 / BLOCK_SIZE as f64).ceil() as usize];
    println!("reading data from {:?}", data_indices);
    let mut left_to_read = inode.size;
    let mut bytes: Vec<u8> = vec![];
    for ind in data_indices {
      let read = std::cmp::min(left_to_read, BLOCK_SIZE);
      left_to_read -= read;
      let mut batch = self.storage.read(self.superblock.data_blocks + ind * BLOCK_SIZE, read)?;
      bytes.append(&mut batch);
    }
    Ok(bytes)
  }

  fn write_data(&mut self, is_directory: bool, data: &[u8]) -> Result<usize> {
    let blocks_needed = (data.len() as f64 / self.superblock.block_size as f64).ceil() as usize;
    if blocks_needed > INODE_LINKS { return Err(anyhow!("The file is too big")) };
    let mut indices: Vec<usize> = vec![];
    for _ in 0..blocks_needed {
      let from = indices.last().unwrap_or(&0);
      let found = self.data_bitmap.find_free_from(*from + 1)
                                  .ok_or(anyhow!("Could not locate enough free datablocks"))?;
      indices.push(found); 
    }
    let mut direct = [0; INODE_LINKS];
    direct.copy_from_slice(&indices[0..indices.len()]);
    let inode = Inode{
      size: data.len(),
      is_directory: is_directory,
      direct: direct,
    };
    let inode_ind = self.write_inode(&inode)?;
    println!("found indices {:?}", indices);
    for (i, &ind) in indices.iter().enumerate() {
      let block_size = self.superblock.block_size;
      let from = i * block_size;
      let to = std::cmp::min(data.len(), from + block_size);
      let write_from = self.superblock.data_blocks + ind * block_size;
      self.storage.write(write_from, &data[from..to])?;
      self.data_bitmap.set(ind, true)?;
    }
    Ok(inode_ind)
  }

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
    let data_bitmap = read_or_new::<DataBitmap>(&mut storage, sb.data_bitmap, BLOCKS_BITMAP_SIZE)?;
    let inode_bitmap = read_or_new::<InodeBitmap>(&mut storage, sb.inode_bitmap, INODES_BITMAP_SIZE)?;

    
    Ok(Fs {
      superblock: sb,
      data_bitmap: data_bitmap,
      inode_bitmap: inode_bitmap,
      storage: storage,
      cur_dir: "/".to_owned(),
    })
  }

  pub fn pwd(&self) -> &String {
    &self.cur_dir
  }

  pub fn touch(&mut self, filename: String, content: &[u8]) -> Result<()> {
    Ok(())
  }

  pub fn mkdir(&mut self, dirname: String) -> Result<()> {
    Ok(())
  }

  pub fn cat(&self, filename: String) -> Result<Vec<u8>> {
    Ok(vec![])
  }

  pub fn ls(&self) -> Result<Vec<&str>> {
    Ok(vec![])
  }

  pub fn rm(&mut self, name: String) -> Result<()> {
    Ok(())
  }

  pub fn cd(&mut self, name: String) -> Result<()> {
    Ok(())
  }
} 