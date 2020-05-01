pub mod structure;
pub mod storage;

use structure::*;
use storage::Storage;

use std::fmt::Debug;
use serde::{Serialize};
use serde::de::DeserializeOwned;

use anyhow::{anyhow, Result, Error};

#[derive(Debug)]
pub struct Fs {
  superblock: Superblock,
  data_bitmap: DataBitmap,
  inode_bitmap: InodeBitmap,
  storage: Storage,
  pub cur_dir: String,
  cur_inode_ind: usize,
}

impl Fs {
  fn read_inode(&self, inode_ind: usize) -> Result<Inode> {
    let bytes = self.storage.read(self.superblock.inode_table + inode_ind * INODE_SIZE, INODE_SIZE)?;
    let inode: Inode = bincode::deserialize(&bytes)?; 
    Ok(inode)
  }

  fn update_inode(&mut self, inode_ind: usize, inode: &Inode) -> Result<()> {
    let bytes = bincode::serialize(inode)?;
    self.storage.write(self.superblock.inode_table + inode_ind * INODE_SIZE, &bytes)?;
    Ok(())
  }

  fn write_new_inode(&mut self, inode: &Inode) -> Result<usize> {
    let ind = self.inode_bitmap.find_free().ok_or(anyhow!("Could not locate free inode"))?;
    let offset = self.superblock.inode_table + ind * self.superblock.inode_size;
    let bytes = bincode::serialize(inode)?;
    self.storage.write(offset, &bytes)?;
    self.inode_bitmap.set(ind, true)?;
    self.dump_inode_bitmap()?;
    Ok(ind)
  }

  fn dump_inode_bitmap(&mut self) -> Result<()> {
    let bytes = bincode::serialize(&self.inode_bitmap)?;
    self.storage.write(self.superblock.inode_bitmap, &bytes)?;
    Ok(())
  }

  fn dump_data_bitmap(&mut self) -> Result<()> {
    let bytes = bincode::serialize(&self.data_bitmap)?;
    self.storage.write(self.superblock.data_bitmap, &bytes)?;
    Ok(())
  }

  fn free_inode(&mut self, inode_ind: usize) -> Result<()> {
    let inode = self.read_inode(inode_ind)?;
    let blocks_taken = (inode.size as f64 / self.superblock.block_size as f64).ceil() as usize; 
    for i in 0..blocks_taken {
      self.data_bitmap.set(inode.direct[i], false)?;
    }
    self.inode_bitmap.set(inode_ind, false)?;
    self.dump_data_bitmap()?;
    self.dump_inode_bitmap()?;
    Ok(())
  }

  fn read_data<T: DeserializeOwned>(&self, inode: &Inode) -> Result<T> {
    let data_indices = &inode.direct[0..(inode.size as f64 / BLOCK_SIZE as f64).ceil() as usize];
    let mut left_to_read = inode.size;
    let mut bytes: Vec<u8> = vec![];
    for ind in data_indices {
      let read = std::cmp::min(left_to_read, BLOCK_SIZE);
      left_to_read -= read;
      let mut batch = self.storage.read(self.superblock.data_blocks + ind * BLOCK_SIZE, read)?;
      bytes.append(&mut batch);
    }
    let data: T = bincode::deserialize(&bytes)?;
    Ok(data)
  }

  fn update_data<T: Serialize>(&mut self, inode: &mut Inode, data: &T) -> Result<()> {
    let data_bytes = bincode::serialize(data)?;
    let blocks_taken = (inode.size as f64 / self.superblock.block_size as f64).ceil() as usize;
    let blocks_needed = (data_bytes.len() as f64 / self.superblock.block_size as f64).ceil() as usize;
    if blocks_needed > INODE_LINKS { return Err(anyhow!("The file is too big")) };
    let mut indices: Vec<usize> = vec![];
    for i in 0..blocks_needed {
      if i < blocks_taken { indices.push(inode.direct[i]) }
      else {
        let from = indices.last().unwrap_or(&0);
        let found = self.data_bitmap.find_free_from(*from + 1)
                                    .ok_or(anyhow!("Could not locate enough free datablocks"))?;
        indices.push(found); 
      }
    }
    if blocks_needed < blocks_taken {
      for &ind in inode.direct[blocks_needed..blocks_taken].iter() {
        self.data_bitmap.set(ind, false)?;
      }
    }
    let mut direct = [0; INODE_LINKS];
    for i in 0..indices.len() {
      direct[i] = indices[i];
    }
    for (i, &ind) in indices.iter().enumerate() {
      let block_size = self.superblock.block_size;
      let from = i * block_size;
      let to = std::cmp::min(data_bytes.len(), from + block_size);
      let write_from = self.superblock.data_blocks + ind * block_size;
      self.storage.write(write_from, &data_bytes[from..to])?;
      self.data_bitmap.set(ind, true)?;
    }
    self.dump_data_bitmap()?;
    inode.size = data_bytes.len();
    inode.direct = direct;
    Ok(())
  }

  fn write_data<T: Serialize>(&mut self, is_directory: bool, data: &T) -> Result<(usize, Inode)> {
    let mut inode = Inode{
      size: 0,
      is_directory: is_directory,
      direct: [0; INODE_LINKS],
    };
    self.update_data(&mut inode, data)?;
    let inode_ind = self.write_new_inode(&inode)?;
    Ok((inode_ind, inode))
  }

  pub fn new(filename: &str) -> Result<Self> {
    let mut storage = Storage::new(filename)?;
    fn read_or_new<T: Default + Serialize + DeserializeOwned + Debug>
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
   
    let mut fs = Fs {
      superblock: sb,
      data_bitmap: data_bitmap,
      inode_bitmap: inode_bitmap,
      storage: storage,
      cur_dir: "/".to_owned(),
      cur_inode_ind: 0,
    };

    let directory = Directory{ parent_inode_ind: None, files: vec![] };
    fs.write_data(true, &directory)?;
    Ok(fs)
  }

  pub fn pwd(&self) -> &String {
    &self.cur_dir
  }

  pub fn new_file<T: Serialize>(&mut self, filename: String, content: &T, is_directory: bool) -> Result<()> {
    let mut cur_dir_inode = self.read_inode(self.cur_inode_ind)?;
    let mut cur_dir: Directory = self.read_data(&cur_dir_inode)?;
    if cur_dir.files.iter().find(|(_, name)| *name == filename).is_some() { 
      return Err(anyhow!("File already exists: {}", filename)) 
    };
    let (data_inode_ind, _) = self.write_data(is_directory, &content)?;
    cur_dir.files.push((data_inode_ind, filename)); 
    self.update_data(&mut cur_dir_inode, &cur_dir)?;
    self.update_inode(self.cur_inode_ind, &cur_dir_inode)?;
    Ok(())
  }

  pub fn touch(&mut self, filename: String, content: &[u8]) -> Result<()> {
    if filename.chars().find(|&c| c == '/').is_some() { 
      return Err(anyhow!("Illegal filename: {}", filename)) 
    }
    self.new_file(filename, &content, false)
  }

  pub fn mkdir(&mut self, name: String) -> Result<()> {
    if name.chars().find(|&c| c == '/').is_some() { 
      return Err(anyhow!("Illegal directory name: {}", name)) 
    }
    let dirname = format!("{}/", name); 
    let directory = Directory{ parent_inode_ind: Some(self.cur_inode_ind), files: vec![] };
    self.new_file(dirname, &directory, true)
  }

  pub fn cat(&self, filename: String) -> Result<String> {
    let cur_dir_inode = self.read_inode(self.cur_inode_ind)?;
    let cur_dir: Directory = self.read_data(&cur_dir_inode)?;
    let &(data_inode_ind, _) = cur_dir.files.iter()
                                            .find(|(_, name)| *name == filename)
                                            .ok_or(anyhow!("Unknown filename: {}", filename))?;
    let data_inode = self.read_inode(data_inode_ind)?;
    let content: String = self.read_data(&data_inode)?; 
    Ok(content)
  }

  pub fn ls(&self) -> Result<Vec<String>> {
    let cur_dir_inode = self.read_inode(self.cur_inode_ind)?;
    let cur_dir: Directory = self.read_data(&cur_dir_inode)?;
    let mut names: Vec<String> = cur_dir.files.iter().map(|(_, name)| format!("{}", name)).collect();
    if cur_dir.parent_inode_ind.is_some() { names.push("..".to_owned()) };
    Ok(names)
  }

  pub fn rm(&mut self, filename: String) -> Result<()> {
    let mut cur_dir_inode = self.read_inode(self.cur_inode_ind)?;
    let mut cur_dir: Directory = self.read_data(&cur_dir_inode)?;
    let (i, &(data_inode_ind, _)) = cur_dir.files.iter().enumerate()
                                            .find(|(_, (_, name))| *name == filename)
                                            .ok_or(anyhow!("Unknown filename: {}", filename))?;
    cur_dir.files.remove(i);  
    self.update_data(&mut cur_dir_inode, &cur_dir)?;
    self.update_inode(self.cur_inode_ind, &cur_dir_inode)?;
    let data_inode = self.read_inode(data_inode_ind)?;
    if data_inode.is_directory {
      let dir: Directory = self.read_data(&data_inode)?;
      for &(inode_ind, _) in dir.files.iter() {
        self.free_inode(inode_ind)?;
      }
    }
    self.free_inode(data_inode_ind)?;
    Ok(())
  }

  pub fn cd(&mut self, name: String) -> Result<()> {
    let dir_inode = self.read_inode(self.cur_inode_ind)?;
    let cur_dir: Directory = self.read_data(&dir_inode)?;
    if name == ".." {
      match cur_dir.parent_inode_ind {
        None => { return Err(anyhow!("Already at root")) },
        Some(inode_ind) => {
          self.cur_inode_ind = inode_ind;
          let mut split: Vec<_> = self.cur_dir.split("/").collect();
          split.remove(split.len() - 2);
          let new_dir = split.join("/");
          self.cur_dir = new_dir;
          return Ok(());
        }
      }
    }
    let dirname = format!("{}/", name); 
    let &(data_inode_ind, _) = cur_dir.files.iter()
                                            .find(|(_, name)| *name == dirname)
                                            .ok_or(anyhow!("Unknown directory name: {}", dirname))?;
    let data_inode = self.read_inode(data_inode_ind)?;
    if !data_inode.is_directory { return Err(anyhow!("Is not a directory: {}", name)) };
    self.cur_dir = format!("{}{}", self.cur_dir, dirname); 
    self.cur_inode_ind = data_inode_ind;
    Ok(())
  }
} 