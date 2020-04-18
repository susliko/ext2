use std::mem::size_of;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub const INODES_COUNT: usize = 1024;
pub const BLOCKS_COUNT: usize = 1024;
pub const BLOCK_SIZE: usize = 1024;
pub const INODE_SIZE: usize = size_of::<Inode>();
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
    let data_blocks = SUPERBLOCK_SIZE + INODE_SIZE * INODES_COUNT;
    Superblock {
      block_size: BLOCK_SIZE, 
      inode_size: INODE_SIZE,
      blocks_count: BLOCKS_COUNT,
      inodes_count: INODES_COUNT, 
      data_bitmap: 0,
      inode_bitmap: 0,
      inode_table: 0,
      data_blocks: 0
    }}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Inode {
  pub size: usize,
  pub direct: [usize; 12],
  pub indirect: usize,
}

impl Default for Inode {
  fn default() -> Self {
    Inode {
      size: 0,
      direct: [0; 12],
      indirect: 0,
    }
  }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Bitmap {
  inner: Vec<u8>
}

impl Bitmap {
  pub fn new(size: usize) -> Self {
    Bitmap {
      inner: vec![0u8; size]
    }
  }

  pub fn set(&mut self, ind: usize, is_taken: bool) -> Result<(), String> {
    let byte = self.inner.get(ind / 8).ok_or(format!("Out of bounds of bitmap: {}", ind))?;
    let shift = 7 - ind % 8;
    let upd_byte = 
      if is_taken { byte | (1 << shift)}
      else { byte & !(1 << shift)};
    println!("{}", format!("{:b}", upd_byte));
    self.inner[ind / 8] = upd_byte;
    Ok(())
  }

  pub fn free_at(&self, ind: usize) -> bool {
    let shift = ind % 8;
    let mask = 1 << 7;
    let byte = self.inner.get(ind / 8).unwrap_or(&0);
    (byte << shift) & mask != mask
  }

  pub fn find_free(&self) -> Option<usize> {
    (0..self.inner.len() * 8).find(|&i| self.free_at(i))
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn set() {
    let mut bitmap = Bitmap{
      inner: vec![0b00000000, 0b00000000]
    };
    bitmap.set(8, true);
    assert_eq!(bitmap.free_at(8), false);
    bitmap.set(8, false);
    assert_eq!(bitmap.free_at(8), true);
    assert_eq!(bitmap.set(20, true).is_err(), true);
  }

  #[test]
  fn free_at() {
    let bitmap = Bitmap{
      inner: vec![0b00000001, 0b10001000]
    };
    assert_eq!(bitmap.free_at(0), true);
    assert_eq!(bitmap.free_at(7), false);
    assert_eq!(bitmap.free_at(8), false);
    assert_eq!(bitmap.free_at(9), true);
    assert_eq!(bitmap.free_at(12), false);
  }

  #[test]
  fn find_free() {
    let bitmap1 = Bitmap{
      inner: vec![0b11100001]
    };
    let bitmap2 = Bitmap{
      inner: vec![0b11111111, 0b11111110]
    };
    let bitmap3 = Bitmap{
      inner: vec![0b11111111, 0b11111111]
    };
    assert_eq!(bitmap1.find_free(), Some(3));
    assert_eq!(bitmap2.find_free(), Some(15));
    assert_eq!(bitmap3.find_free(), None); 
  }
}