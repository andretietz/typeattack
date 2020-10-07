use crate::arguments::Arguments;
use crate::graphics::{Crossterm, RenderEngine};

pub struct State;

pub struct Typeattack<T: RenderEngine> {
  state: State,
  level: u16,
  words: Vec<String>,
  engine: T,
}

pub enum Event {
  Pause,
  Key(char),
}

impl Typeattack<Crossterm> {
  pub fn new(args: &Arguments) -> Self {
    return Typeattack {
      state: State,
      level: args.level,
      // TODO: not ideal, fine for now
      words: args.file.clone(),
      engine: Crossterm::new(),
    };
  }

  pub fn run(self: &Self) {
    let mut end_condition = true;
    self.engine.clear_screen();
    while end_condition {}
    self.engine.clear_screen();
  }
}
