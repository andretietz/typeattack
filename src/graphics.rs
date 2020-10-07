use crate::typeattack::Event;
use crossterm::{
  cursor, event,
  event::{poll, read, KeyCode},
  execute,
  style::{self, Colorize},
  terminal, ExecutableCommand, QueueableCommand, Result,
};
use genawaiter::{rc::gen, yield_};
use std::io::{stdout, Stdout, Write};
use std::time::Duration;

pub trait RenderEngine {
  fn clear_screen(self: &Self);
  // some stream of type Event
  fn event_stream(self: &Self);
}

pub struct Crossterm {}

impl Crossterm {
  pub fn new() -> Self {
    Crossterm {}
  }
}

impl RenderEngine for Crossterm {
  fn clear_screen(self: &Self) {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All));
  }

  fn event_stream(self: &Self) {
    let generator = gen!({
      let mut end_condition = true;
      while end_condition {
        if poll(Duration::from_millis(500)).unwrap() {
          match read().unwrap() {
            event::Event::Key(event) => {
              if event.code == KeyCode::Esc {
                yield_!(Ok(Event::Pause));
                end_condition = false;
              } else {
                yield_!(Ok(Event::Key('*')));
              }
            }
            event::Event::Mouse(event) => println!("{:?}", event),
            event::Event::Resize(width, height) => println!("New size {}x{}", width, height),
          }
        } else {
          // Timeout expired and no `Event` is available
        }
      }
      Ok("")
    });
  }
}
