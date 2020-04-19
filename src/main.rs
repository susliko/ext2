pub mod fs;
use fs::Fs;
use std::io::{self};
use std::io::prelude::*;
use std::str::FromStr;
use anyhow::Error;

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

const HELP_MESSAGE: &str = 
"pwd              - prints active directory
ls               - lists all filenames in active directory
exit             - exits the application
help             - prints this message
cd    [dest]     - sets active directory to `dest`
touch [filename] - creates a new file with content of the next entered line
mkdir [dirname]  - creates a new directory
cat   [filename] - prints the content of the file
rm    [name]     - removes file or directory";

fn main() {
  fn on_error(e: Error) { println!("{:?}", e) };
  let mut fs = Fs::new("index.php").unwrap();
  println!("Welcome to a modest ext2-like file system!. Type `help` to list its capabilities.");
  loop {
    print!("{} > ", fs.cur_dir);
    io::stdout().flush().ok().expect("Could not flush stdout");
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
      Err(why) => println!("Error while reading input: {}", why),
      Ok(_) => {
        buffer.pop();
        match buffer.parse::<Command>() {
          Ok(Command::Pwd) => println!("{}", fs.pwd()),
          Ok(Command::Ls) => 
            match fs.ls() {
              Err(why) => println!("{}", why),
              Ok(names) => println!("{}", names.join("\n")),
            }
          Ok(Command::Exit) => { break },
          Ok(Command::Help) => { println!("{}", HELP_MESSAGE) },
          Ok(Command::Cd(dest)) => 
            { fs.cd(dest).map_or_else(on_error, |_| ()) },
          Ok(Command::Touch(name)) => { 
              let mut buffer = String::new();
              match io::stdin().read_line(&mut buffer) {
                Err(why) => { println!("Error while reading input: {}", why); continue }
                Ok(_) => {
                  buffer.pop();
                  fs.touch(name, buffer.as_bytes()).map_or_else(on_error, |_| ()) },
              }
            }
          Ok(Command::Mkdir(name)) =>
            { fs.mkdir(name).map_or_else(on_error, |_| ()) },
          Ok(Command::Cat(name)) =>
            { fs.cat(name).map_or_else(on_error, |content| println!("{}", content))},
          Ok(Command::Rm(name)) =>
            { fs.rm(name).map_or_else(on_error, |_| ())},
          Err(why) => println!("{:?}", why),
       }
      },
    }
  }
}
