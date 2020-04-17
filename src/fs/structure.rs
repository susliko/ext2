use std::mem::size_of;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct Superblock {
  pub block_size: u32,
  pub inode_size: u32,
  pub blocks_count: u32,
  pub inodes_count: u32,
}

impl Default for Superblock {
  fn default() -> Self { 
    Superblock {
      block_size: 1024, 
      inode_size: size_of::<Inode>() as u32,
      blocks_count: 1024,
      inodes_count: 1024, 
    }}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Inode {
  pub size: i32,
  pub direct: [u32; 12],
  pub indirect: u32,
}

impl Default for Inode {
  fn default() -> Self {
    Inode {
      size: 0,
      direct: [0u32; 12],
      indirect: 0,
    }
  }
}