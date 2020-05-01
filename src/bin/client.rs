use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, BufWriter};
use std::net::TcpStream;
use std::thread;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;
use std::str;
use std::env;

fn main() -> std::io::Result<()> {
  let mut host = "127.0.0.1";
  let mut port = "4242";
  let args: Vec<_> = env::args().collect();
  match args.as_slice() {
    [_, env_host, env_port] => { host = env_host; port = env_port },
    [_, env_port] => { port = env_port },
    [_] => {},
    _ => { println!("Possible arguments: [optional host] [optional port]"); return Ok(()) }
  }
  let address = format!("{}:{}", host, port);
  println!("connecting to {}", address);

  let stream = TcpStream::connect(address)?;
  let mut reader = BufReader::new(stream.try_clone()?);
  let mut writer = BufWriter::new(stream);
  let cond = Arc::new(AtomicBool::new(true));
  let read_cond = cond.clone();
  let write_cond = cond.clone();

  let writer_thread = thread::spawn(move || {
    let mut buffer = String::new();
    while write_cond.load(Ordering::SeqCst) {
      stdin().read_line(&mut buffer).ok();
      writer.write(&buffer.as_bytes()).ok();
      writer.flush().ok();
      buffer = "".to_owned();
    }
  }); // thread which endlessly captures stdin and sends it to the socket

  let reader_thread = thread::spawn(move || {
    let mut buffer = [0u8, 32];
    while let Ok(size) = reader.read(&mut buffer) { 
      if size == 0 { break };
      let s = str::from_utf8(&buffer).unwrap_or("");
      print!("{}", s);
      buffer = [0u8, 32];
      stdout().flush().ok();
    };
    read_cond.store(false, Ordering::SeqCst);
    println!("Press Enter to exit");
  }); // thread which endlessly fetches data from the socket and prints it to stdout

  writer_thread.join().ok();
  reader_thread.join().ok();

  Ok(())
}
