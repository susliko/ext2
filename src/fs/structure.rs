use serde::{Deserialize, Serialize};
use serde_big_array::big_array;
use std::fmt::Debug;
use std::mem::size_of;
use anyhow::{anyhow, Result};

pub const INODES_COUNT: usize = 1024;
pub const INODE_SIZE: usize = size_of::<Inode>();
pub const INODES_BITMAP_SIZE: usize = INODES_COUNT / 8;
pub const INODE_LINKS: usize = 12;

pub const BLOCKS_COUNT: usize = 1024;
pub const BLOCK_SIZE: usize = 1024;
pub const BLOCKS_BITMAP_SIZE: usize = BLOCKS_COUNT / 8;

pub const SUPERBLOCK_SIZE: usize = size_of::<Superblock>();

#[derive(Serialize, Deserialize, Debug)]
pub struct Superblock {
  pub block_size: usize,
  pub inode_size: usize,
  pub blocks_count: usize,
  pub inodes_count: usize,
  pub data_bitmap: usize,
  pub inode_bitmap: usize,
  pub inode_table: usize,
  pub data_blocks: usize,
}

impl Default for Superblock {
  fn default() -> Self {
    Superblock {
      block_size: BLOCK_SIZE,
      inode_size: INODE_SIZE,
      blocks_count: BLOCKS_COUNT,
      inodes_count: INODES_COUNT,
      data_bitmap: SUPERBLOCK_SIZE,
      inode_bitmap: SUPERBLOCK_SIZE + BLOCKS_BITMAP_SIZE,
      inode_table: SUPERBLOCK_SIZE + BLOCKS_BITMAP_SIZE + INODES_BITMAP_SIZE,
      data_blocks: SUPERBLOCK_SIZE
        + BLOCKS_BITMAP_SIZE
        + INODES_BITMAP_SIZE
        + INODES_COUNT * INODE_SIZE,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Inode {
  pub size: usize,
  pub is_directory: bool, 
  pub direct: [usize; INODE_LINKS],
}

impl Default for Inode {
  fn default() -> Self {
    Inode {
      size: 0,
      is_directory: false,
      direct: [0; 12],
    }
  }
}

// Custom bitmap abstraction with handy methods
pub trait Bitmap<'a> {
  fn mutable(&'a mut self) -> &'a mut [u8];
  fn immutable(&'a self) -> &'a [u8];

  fn set(&'a mut self, ind: usize, is_taken: bool) -> Result<()> {
    let inner = self.mutable();
    let byte = inner.get(ind / 8)
      .ok_or(anyhow!("Out of bounds of bitmap: {}", ind))?;
    let shift = 7 - ind % 8;
    let upd_byte = if is_taken {
      byte | (1 << shift)
    } else {
      byte & !(1 << shift)
    };
    inner[ind / 8] = upd_byte;
    Ok(())
  }

  fn free_at(&'a self, ind: usize) -> bool {
    let shift = ind % 8;
    let mask = 1 << 7;
    let byte = self.immutable().get(ind / 8).unwrap_or(&0);
    (byte << shift) & mask != mask
  }

  fn find_free_from(&'a self, from: usize) -> Option<usize> {
    (from..self.immutable().len() * 8).find(|&i| self.free_at(i))
  }

  fn find_free(&'a self) -> Option<usize> { self.find_free_from(0) }
}

big_array! { BigArray; }
#[derive(Serialize, Deserialize)]
pub struct InodeBitmap {
  #[serde(with = "BigArray")]
  inner: [u8; INODES_BITMAP_SIZE],
}

impl<'a> Bitmap<'a> for InodeBitmap {
  fn mutable(&'a mut self) -> &'a mut [u8] { &mut self.inner }
  fn immutable(&'a self) -> &'a [u8] { &self.inner }
}

impl Default for InodeBitmap {
  fn default() -> Self { InodeBitmap { inner: [0; INODES_BITMAP_SIZE] } }
}

impl Debug for InodeBitmap {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner[..].fmt(formatter)
  }
}

#[derive(Serialize, Deserialize,)]
pub struct DataBitmap {
  #[serde(with = "BigArray")]
  inner: [u8; BLOCKS_BITMAP_SIZE],
}

impl Default for DataBitmap {
  fn default() -> Self { DataBitmap { inner: [0; INODES_BITMAP_SIZE] } }
}

impl Debug for DataBitmap {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.inner[..].fmt(formatter)
  }
}

impl<'a> Bitmap<'a> for DataBitmap {
  fn mutable(&'a mut self) -> &'a mut [u8] { &mut self.inner }
  fn immutable(&'a self) -> &'a [u8] { &self.inner}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn set() {
    let mut bitmap: InodeBitmap = Default::default();
    bitmap.set(8, true).unwrap();
    assert_eq!(bitmap.free_at(8), false);
    bitmap.set(8, false).unwrap();
    assert_eq!(bitmap.free_at(8), true);
    assert_eq!(bitmap.set(2000, true).is_err(), true);
  }

  #[test]
  fn free_at() {
    let mut bitmap: InodeBitmap = Default::default();
    bitmap.inner[0] = 0b00000001; bitmap.inner[1] = 0b10001000;
    assert_eq!(bitmap.free_at(0), true);
    assert_eq!(bitmap.free_at(7), false);
    assert_eq!(bitmap.free_at(8), false);
    assert_eq!(bitmap.free_at(9), true);
    assert_eq!(bitmap.free_at(12), false);
  }

  #[test]
  fn find_free() {
    let mut bitmap1: InodeBitmap = Default::default();
    bitmap1.inner[0] = 0b11100001;
    let mut bitmap2: InodeBitmap = Default::default();
    bitmap2.inner[0] = 0b11111111; bitmap2.inner[1] = 0b11111110;
    let mut bitmap3: InodeBitmap = Default::default();
    for b in bitmap3.inner.iter_mut() { *b = 0b11111111 };
    assert_eq!(bitmap1.find_free(), Some(3));
    assert_eq!(bitmap2.find_free(), Some(15));
    assert_eq!(bitmap3.find_free(), None);
  }
}
