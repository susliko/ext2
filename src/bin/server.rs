use ::fs::Fs;
use anyhow::Error;
use daemonize::Daemonize;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

#[derive(Debug)]
enum Command {
  Pwd,
  Ls,
  Help,
  Exit,
  Cd(String),
  Touch(String),
  Mkdir(String),
  Cat(String),
  Rm(String),
}

impl FromStr for Command {
  type Err = String;

  fn from_str(s: &str) -> Result<Command, Self::Err> {
    let split: Vec<&str> = s.split(" ").collect();
    match split.as_slice() {
      ["pwd"] => Ok(Command::Pwd),
      ["ls"] => Ok(Command::Ls),
      ["exit"] => Ok(Command::Exit),
      ["help"] => Ok(Command::Help),
      ["cd", dest] => Ok(Command::Cd((*dest).to_owned())),
      ["touch", name] => Ok(Command::Touch((*name).to_owned())),
      ["mkdir", name] => Ok(Command::Mkdir((*name).to_owned())),
      ["cat", name] => Ok(Command::Cat((*name).to_owned())),
      ["rm", name] => Ok(Command::Rm((*name).to_owned())),
      x => Err("Unknown command: ".to_owned() + x.get(0).unwrap_or(&"")),
    }
  }
}

const HELP_MESSAGE: &str = "pwd              - prints active directory
ls               - lists all filenames in active directory
exit             - exits the application
help             - prints this message
cd    [dest]     - sets active directory to `dest`
touch [filename] - creates a new file with content of the next entered line
mkdir [dirname]  - creates a new directory
cat   [filename] - prints the content of the file
rm    [name]     - removes file or directory";

fn handle_client(fs: &mut Fs, stream: &mut TcpStream) -> std::io::Result<()> {
  let mut reader = BufReader::new(stream.try_clone()?);
  let mut writer = BufWriter::new(stream);
  writer.write(
    b"Welcome to a modest ext2-like file system!. Type `help` to list its capabilities.\n",
  )?;
  fn write_err<R: Write>(writer: &mut BufWriter<R>, err: Error) {
    writer.write(format!("{:?}\n", err).as_bytes()).ok();
    ()
  }
  fn write_msg<R: Write>(writer: &mut BufWriter<R>, msg: &String) {
    writer.write(format!("{}\n", msg).as_bytes()).ok();
    ()
  }

  loop {
    writer.write(format!("{} > ", fs.cur_dir).as_bytes())?;
    writer.flush().ok();
    let mut buffer = String::new();
    match reader.read_line(&mut buffer) {
      Err(why) => write_msg(&mut writer, &format!("Error while reading input: {}", why)),
      Ok(_) => {
        let buffer = buffer.replace(&['\n', '\r'][..], "");
        match buffer.parse::<Command>() {
          Ok(Command::Pwd) => write_msg(&mut writer, &format!("{}", fs.pwd())),
          Ok(Command::Ls) => match fs.ls() {
            Err(why) => write_err(&mut writer, why),
            Ok(names) => write_msg(&mut writer, &format!("{}", names.join("\n"))),
          },
          Ok(Command::Exit) => break,
          Ok(Command::Help) => write_msg(&mut writer, &format!("{}", HELP_MESSAGE)),
          Ok(Command::Cd(dest)) => fs
            .cd(dest)
            .map_or_else(|e| write_err(&mut writer, e), |_| ()),
          Ok(Command::Touch(name)) => {
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
              Err(why) => write_msg(&mut writer, &format!("Error while reading input: {}", why)),
              Ok(_) => {
                buffer.pop();
                fs.touch(name, buffer.as_bytes())
                  .map_or_else(|e| write_err(&mut writer, e), |_| ())
              }
            }
          }
          Ok(Command::Mkdir(name)) => fs
            .mkdir(name)
            .map_or_else(|e| write_err(&mut writer, e), |_| ()),
          Ok(Command::Cat(name)) => {
            let msg = fs
              .cat(name)
              .map_or_else(|e| format!("{}", e), |content| content);
            write_msg(&mut writer, &msg)
          }
          Ok(Command::Rm(name)) => fs
            .rm(name)
            .map_or_else(|e| write_err(&mut writer, e), |_| ()),
          Err(why) => write_msg(&mut writer, &format!("{}", why)),
        }
      }
    }
  }
  Ok(())
}

fn bind_and_handle(port: &str) -> std::io::Result<()> {
  let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
  let mut fs = Fs::new("index.php").unwrap();
  for stream in listener.incoming() {
    println!("Client entered");
    handle_client(&mut fs, &mut stream?).map_or_else(
      |e| println!("Error while communicating with client: {}", e),
      |_| (),
    );
    println!("Client left");
  }
  Ok(())
}

fn main() {
  let mut port = "4242";
  let args: Vec<_> = env::args().collect();
  match args.as_slice() {
    [_, env_port] => port = env_port,
    [_] => {}
    _ => {
      println!("Possible arguments: [optional port]");
      return ();
    }
  }
  println!("Running file system daemon on port {}", port);

  let logs_path = "/tmp/daemon.out";
  let error_logs_path = "/tmp/daemon.err";
  let stdout = File::create("/tmp/daemon.out").unwrap();
  let stderr = File::create("/tmp/daemon.err").unwrap();
  println!("Writing logs to {} and {}", logs_path, error_logs_path);

  let daemonize = Daemonize::new()
    .pid_file("/tmp/ext2server.pid")
    .working_directory("/tmp")
    .stdout(stdout)
    .stderr(stderr);

  match daemonize.start() {
    Ok(_) => println!("Running as daemon"),
    Err(e) => eprintln!("Error running as daemon: {}", e),
  }

  bind_and_handle(port).map_or_else(|e| println!("Error while handling socket: {:?}", e), |_| ())
}
