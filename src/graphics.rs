use std::io::{stdout, Write};
use crossterm::{
  cursor::{Hide, MoveTo, RestorePosition, SavePosition},
  event::{self, KeyCode},
  execute,
  queue, style::Print, terminal::{Clear, ClearType, enable_raw_mode},
};
use futures::{
  stream::Stream, stream::StreamExt,
};
use crate::typeattack::{Event, Word, WorldState};
use std::pin::Pin;

pub trait RenderEngine {
  fn clear_screen(self: &Self);
  // some stream of type Event
  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>>;

  fn update(self: &Self, state: &WorldState, old: &WorldState);
}

pub struct Crossterm {
  size_x: u16,
  size_y: u16,
}

impl Crossterm {
  pub fn new() -> Self {
    // let (x, y) = terminal::size().unwrap();
    Crossterm {
      size_x: 10,
      size_y: 10,
    }
  }

  fn get_position(self: &Self, word: &Word) -> (u16, u16) {
    let y = self.size_y as f64 * word.y;
    let x = self.size_x as f64 * word.x;
    (x.round() as u16, y.round() as u16)
  }

  fn stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>> {
     return event::EventStream::new()
        // filter all events we don't require
        .filter(|result| {
          return futures::future::ready(match result {
            Ok(event) => {
              match event {
                event::Event::Key(key) => {
                  match key.code {
                    KeyCode::Esc => true,
                    KeyCode::Char(_) => true,
                    _ => false
                  }
                }
                _ => false
              }
            }
            _ => false
          })
        })
        .map(|result| {
          return match result {
            Ok(event) => {
              match event {
                event::Event::Key(key) => {
                  match key.code {
                    KeyCode::Esc => Event::Pause,
                    KeyCode::Char(c) => Event::Key(c),
                    _ => panic!("")
                  }
                }
                _ => panic!("")
              }
            }
            _ => panic!("")
          }
        }).boxed();
  }
}

impl RenderEngine for Crossterm {
  fn clear_screen(self: &Self) {
    enable_raw_mode().unwrap();
    execute!(stdout(), Clear(ClearType::All), Hide).unwrap();
  }

  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>> {
    self.stream()
  }


  fn update(self: &Self, state: &WorldState, old: &WorldState) {
    queue!(stdout(), SavePosition).unwrap();
    // remove old words
    for word in &old.words {
      let (x, y) = self.get_position(&word);
      queue!(
        stdout(),
        MoveTo(x, y),
        Print(" ".repeat(word.word.len()))
      ).unwrap();
    }
    // update new words
    for word in &state.words {
      let (x, y) = self.get_position(&word);
      queue!(
        stdout(),
        MoveTo(x, y),
        Print(&word.word)
      ).unwrap();
    }
    queue!(stdout(), RestorePosition).unwrap();
    // apply
    stdout().flush().unwrap();
  }
}
