use std::io::{Stdout, stdout};
use crossterm::{
  cursor, event::{Event, KeyCode, poll, read},
  ExecutableCommand, execute, QueueableCommand, Result,
  style::{self, Colorize},
  terminal,
};

pub trait RenderEngine {
  fn clear_screen(self: &Self);
  // some stream of type Event
  fn event_stream(self: &Self);
}

pub struct Crossterm {
}

impl Crossterm {
  pub fn new() -> Self {
    Crossterm{}
  }
}

impl RenderEngine for Crossterm {
  fn clear_screen(self: &Self) {
    execute!(stdout, terminal::Clear(terminal::ClearType::All));
  }

  fn event_stream(self: &Self) {

  }
}