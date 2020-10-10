use std::io::{stdout, Stdout, Write};
use std::time::Duration;
use crossterm::{
  cursor::{Hide, MoveTo, RestorePosition, SavePosition},
  event::{self, KeyCode, KeyEvent, poll, read},
  ExecutableCommand,
  execute,
  queue, QueueableCommand, Result, style::Colorize, style::Print, terminal::{self, Clear, ClearType},
};
use crossterm::event::EventStream;
use futures::{Stream, StreamExt};
use futures::stream::{Filter, Map};

use crate::typeattack::{Event, Word, WorldState};

pub trait RenderEngine {
  fn clear_screen(self: &Self);
  // some stream of type Event
  fn event_stream(self: &Self) -> Box<dyn Stream<Item=Event>>;

  fn update(self: &Self, state: &WorldState, old: &WorldState);
}

pub struct Crossterm {
  size_x: u16,
  size_y: u16,
}

impl Crossterm {
  pub fn new() -> Self {
    let (x, y) = terminal::size().unwrap();
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

  fn stream(self: &Self) -> impl Stream<Item=Event> {
    return event::EventStream::new()
        // filter all events we don't require
        .filter(|result| self.filter_events(result))
        .map(|result| self.map_events(result));
  }

  fn filter_events(self: &Self, result: &Result<event::Event>) -> bool {
    match result {
      Ok(event) => {
        if let key = (event as KeyEvent).code {
          match key {
            KeyCode::Esc => true,
            KeyCode::Char(c) => true,
            _ => false
          }
        }
        false
      }
      _ => false
    }
  }

  fn map_events(self: &Self, result: Result<event::Event>) -> Event {
    let event = result.unwrap() as KeyEvent;
    match event.code {
      KeyCode::Esc => Event::Pause,
      KeyCode::Char(a) => Event::Key(a),
      _ => panic!("Should be filtered already!")
    }
  }
}

impl RenderEngine for Crossterm {
  fn clear_screen(self: &Self) {
    execute!(stdout(), Clear(ClearType::All), Hide );
  }

  fn event_stream(self: &Self) -> Box<dyn Stream<Item=Event>> {
    Box::new(self.stream())
  }


  fn update(self: &Self, state: &WorldState, old: &WorldState) {
    queue!(stdout(), SavePosition);
    for word in &old.words {
      let (x, y) = self.get_position(&word);
      queue!(
        stdout(),
        MoveTo(x, y),
        Print(" ".repeat(word.word.len()))
      );
    }
    for word in &state.words {
      let (x, y) = self.get_position(&word);
      queue!(
        stdout(),
        MoveTo(x, y),
        Print(&word.word)
      );
    }
    queue!(stdout(), RestorePosition);
    stdout().flush();
  }
}
