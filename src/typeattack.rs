use crate::arguments::Arguments;
use crate::graphics::{Crossterm, RenderEngine};

pub struct State;

pub struct Typeattack<T : RenderEngine> {
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
    while end_condition {
      // `poll()` waits for an `Event` for a given time period
      // if poll(Duration::from_millis(500)).unwrap() {
      //   // It's guaranteed that the `read()` won't block when the `poll()`
      //   // function returns `true`
      //   match read().unwrap() {
      //     Event::Key(event) => {
      //       if event.code == KeyCode::Esc {
      //         end_condition = false;
      //       } else {
      //         println!("{:?}", event);
      //       }
      //     }
      //     Event::Mouse(event) => println!("{:?}", event),
      //     Event::Resize(width, height) => println!("New size {}x{}", width, height),
      //   }
      // } else {
      //   // Timeout expired and no `Event` is available
      // }
    }
    self.engine.clear_screen();
  }
}