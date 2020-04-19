pub mod fs;
use fs::Fs;
use std::io::{self};
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
enum Command {
  Pwd,
  Ls,
  Help,
  Exit,
  Cd(String),
  Touch(String, String),
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
      ["touch", name, content] => Ok(Command::Touch((*name).to_owned(), (*content).to_owned())),
      ["mkdir", name] => Ok(Command::Mkdir((*name).to_owned())),
      ["cat", name] => Ok(Command::Cat((*name).to_owned())),
      ["rm", name] => Ok(Command::Rm((*name).to_owned())),
      x => Err("Unknown command: ".to_owned() + x.get(0).unwrap_or(&"")),
    }
  }
}
fn main() {
  let mut fs = Fs::new("index.php").unwrap();

  println!("Welcome to a modest ext2-like file system!. Type `help` to list its capabilities.");
  println!("{:?}", fs);
  loop {
    print!("> ");
    io::stdout().flush().ok().expect("Could not flush stdout");
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
      Err(why) => println!("Error while reading input: {}", why),
      Ok(_s) => {
        buffer.pop();
        match buffer.parse::<Command>() {
          Ok(Command::Pwd) => println!("{}", fs.pwd()),
          Ok(Command::Ls) => 
            match fs.ls() {
              Err(why) => println!("{}", why),
              Ok(names) => println!("{:?}", names),
            }
          Ok(Command::Exit) => { break },
          Ok(Command::Help) => { println!("useful help") },
          Ok(Command::Cd(dest)) => 
            { fs.cd(dest).map_or((), |e| println!("{:?}", e)) },
          Ok(Command::Touch(name, content)) =>
            { fs.touch(name, content.as_bytes()).map_or((), |e| println!("{:?}", e)) },
          Ok(Command::Mkdir(name)) =>
            { fs.mkdir(name).map_or((), |e| println!("{:?}", e)) },
          Ok(Command::Cat(name)) =>
            { fs.cat(name).map_or((), |e| println!("{:?}", e)) },
          Ok(Command::Rm(name)) =>
            { fs.rm(name).map_or((), |e| println!("{:?}", e))},
          Err(why) => println!("{:?}", why),
       }
      },
    }
  }
}
