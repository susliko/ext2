pub mod fs;
use fs::Fs;
use std::io::{self, Read};
use std::io::prelude::*;
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
fn main() {
  let fs = Fs::new("index.php");

  println!("Welcome to a modest ext2-like file system!. Type `help` to list its capabilities.");
  println!("{:?}", fs);
  loop {
    print!("> ");
    io::stdout().flush().ok().expect("Could not flush stdout");
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
      Err(why) => println!("Error occured: {}", why),
      Ok(_s) => {
        buffer.pop();
        match buffer.parse::<Command>() {
          Ok(command) => println!("{:?}", command),
          Err(why) => println!("{:?}", why),
       }
      },
    }
  }
}
