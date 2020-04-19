use std::cell::RefCell;
use std::fs::*;
use std::io;
use std::io::*;

#[derive(Debug)]
pub struct Storage {
  file: RefCell<File>,
}

impl Storage {
  pub fn new(filename: &str) -> io::Result<Self> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(filename)?;
    Ok(Storage {
      file: RefCell::new(file),
    })
  }

  pub fn write(&mut self, offset: usize, bytes: &[u8]) -> io::Result<usize> {
    self.file.borrow_mut().seek(SeekFrom::Start(offset as u64))?;
    self.file.borrow_mut().write(bytes)
  }

  pub fn read(&self, offset: usize, size: usize) -> io::Result<Vec<u8>> {
    self.file.borrow_mut().seek(SeekFrom::Start(offset as u64))?;
    let mut buffer = vec![0u8; size as usize];
    self.file.borrow_mut().read(buffer.as_mut_slice()).and_then(|total| {
      if total == size { Ok(buffer)}
      else { Err(Error::new(ErrorKind::UnexpectedEof, "unexpected end of file")) }
    })
  }
}