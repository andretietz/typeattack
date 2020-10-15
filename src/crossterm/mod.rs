
use std::io::{stdout, Write};
use crossterm::{
  cursor::{MoveLeft, MoveTo, RestorePosition, SavePosition},
  event::{self, KeyCode},
  execute,
  queue,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType, enable_raw_mode, size},
};

use std::pin::Pin;

use futures::stream::{Stream, StreamExt};

use crate::typeattack::{Event, Word, WorldState, RenderEngine};

pub struct Crossterm {
  size_x: u16,
  size_y: u16,
}

impl Crossterm {
  pub fn new() -> Self {
    let (x, y) = size().unwrap();
    Crossterm {
      size_x: x,
      size_y: y,
    }
  }


  fn print_word(self: &Self, buffer: &String, word: &String) {
    queue!(stdout(), Print(word)).unwrap();
    if word.starts_with(buffer.as_str()) {
      queue!(
        stdout(),
        MoveLeft(word.len() as u16),
        SetForegroundColor(Color::Black), // TODO: invert
        SetBackgroundColor(Color::White), // TODO: invert
        Print(buffer),
        ResetColor
      ).unwrap();
    }
  }

  fn get_position(self: &Self, word: &Word) -> (u16, u16) {
    let y = self.size_y as f64 * word.y;
    let x = self.size_x as f64 * word.x;
    (x.round() as u16, y.round() as u16)
  }

  fn stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>> {
    return event::EventStream::new()
        // filter all events we don't need
        .filter(|result| {
          return futures::future::ready(match result {
            Ok(event) => {
              match event {
                event::Event::Key(key) => {
                  match key.code {
                    KeyCode::Esc => true,
                    KeyCode::Backspace => true,
                    KeyCode::Char(_) => true,
                    _ => false
                  }
                }
                event::Event::Resize(_, _) => true,
                _ => false
              }
            }
            _ => false
          });
        })
        .map(|result| {
          return match result {
            Ok(event) => {
              match event {
                event::Event::Key(key) => {
                  match key.code {
                    KeyCode::Esc => Event::Pause,
                    KeyCode::Backspace => Event::RemoveChar,
                    KeyCode::Char(c) => Event::AddChar(c),
                    _ => panic!("")
                  }
                }
                event::Event::Resize(x, y) => Event::Resize(x, y),
                _ => panic!("")
              }
            }
            _ => panic!("")
          };
        })
        .boxed();
  }
}

impl RenderEngine for Crossterm {
  fn clear_screen(self: &Self) {
    enable_raw_mode().unwrap();
    execute!(stdout(), Clear(ClearType::All)).unwrap();
  }

  fn set_screen_size(self: &mut Self, x: u16, y: u16) {
    self.size_x = x;
    self.size_y = y;
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
        MoveTo(x, y)
      ).unwrap();
      self.print_word(&state.buffer, &word.word);
    }

    // draw HUD
    queue!(stdout(),
      MoveTo(0, self.size_y),
      Print(" ".repeat(self.size_x.into())),
      MoveTo(0, self.size_y),
      Print(format!("Inputs: {} Fails: {} Words: {} Buffer: {}", &state.keycount, &state.fails, &state.wordcount, &state.buffer))
      ).unwrap();
    queue!(stdout(), RestorePosition).unwrap();
    // apply
    stdout().flush().unwrap();
  }
}