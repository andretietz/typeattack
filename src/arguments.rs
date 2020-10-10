use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use clap::{App, AppSettings, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAME: &'static str = env!("CARGO_PKG_NAME");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug)]
pub struct Arguments {
  pub file: Vec<String>,
  pub level: u8,
}

impl Arguments {
  pub fn get() -> Result<Self, String> {
    let app = App::new(NAME)
      .setting(AppSettings::ArgRequiredElseHelp)
      .version(VERSION)
      .author(AUTHORS)
      .about(DESCRIPTION)
      .arg(
        Arg::with_name("wordlistfile")
          .required(true)
          .index(1)
          .value_name("WORDLIST")
          .help("Path to a list of words you want to use."),
      )
      .arg(
        Arg::with_name("level")
          .required(false)
          .index(2)
          .value_name("LEVEL")
          .help("Defines the speed of the words from 1 to 10. Default is 1."),
      );
    let matches = app.get_matches();
    let wordlist_arg = matches.value_of("wordlistfile").unwrap();
    let wordlist = Path::new(wordlist_arg);
    if !wordlist.exists() || !wordlist.is_file() {
      return Err(format!("Couldn\'t find wordlist file: {}", wordlist_arg));
    }
    let level: u8 = matches
      .value_of("level")
      .unwrap_or("1")
      .parse::<u8>()
      .map_err(|_| "Error while parsing level!")?;
    if let Ok(lines) = Self::read_lines(wordlist_arg) {
      return Ok(Arguments {
        file: lines.collect::<Result<_, _>>().unwrap(),
        level,
      });
    }

    return Err(format!("Couldn\'t read words from wordlist file: {}", wordlist_arg));
  }

  fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
  where
    P: AsRef<Path>,
  {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
  }
}
