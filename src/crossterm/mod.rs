use std::io::{stdout, Write};
use std::pin::Pin;

use crossterm::{
  cursor::{MoveTo, RestorePosition, SavePosition},
  event::{self, KeyCode},
  execute,
  queue,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType, enable_raw_mode, size},
};
use futures::stream::{Stream, StreamExt};

use crate::typeattack::{Event, RenderEngine, Word, WorldState};

pub struct Crossterm {
  size_x: u16,
  size_y: u16,
  unit_x: f64,
  unit_y: f64,
}

impl Crossterm {
  pub fn new() -> Self {
    let (x, y) = size().unwrap();
    Crossterm::new_with_size(x, y)
  }

  pub fn new_with_size(x: u16, y: u16) -> Self {
    Crossterm {
      size_x: x,
      size_y: y,
      unit_x: 1.0 / x as f64,
      unit_y: 1.0 / y as f64,
    }
  }


  fn print_word(self: &Self, buffer: &String, word: &Word) {
    queue!(stdout(), Print(&word.word)).unwrap();

    if word.word.starts_with(buffer.as_str()) {
      let (x, y) = self.get_position(word);
      queue!(
        stdout(),
        MoveTo(x, y),
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::White),
        Print(buffer),
        ResetColor
      ).unwrap();
    }
  }

  fn get_position(self: &Self, word: &Word) -> (u16, u16) {
    let word_size = word.word.len() as f64 * self.unit_x;
    let max = (self.size_x as f64 * self.unit_x) - word_size;
    // 1/max = value/x
    // x = max*value/1
    let x = max * (self.size_x as f64 * word.x) / 1.0;
    let y = self.size_y as f64 * word.y;
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
    execute!(stdout(),
      Clear(ClearType::All),
      SetForegroundColor(Color::White),
      SetBackgroundColor(Color::Black),
    ).unwrap();
  }

  fn set_screen_size(self: &mut Self, x: u16, y: u16) {
    self.size_x = x;
    self.size_y = y;
    self.unit_x = 1.0 / x as f64;
    self.unit_y = 1.0 / y as f64;
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
      self.print_word(&state.buffer, &word);
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

#[cfg(test)]
mod tests {
  use crate::crossterm::Crossterm;
  use crate::typeattack::Word;

  /// 0123456789
              /// TEST......
  #[test]
  fn text_left_even() {
    let crossterm = Crossterm::new_with_size(10, 10);
    let word = Word::new("TEST", 0., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
  }

  /// 0123456789
  /// ......TEST
  #[test]
  fn text_right_even() {
    let crossterm = Crossterm::new_with_size(10, 10);
    let word = Word::new("TEST", 1., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 6);
    assert_eq!(y, 0);
  }

  /// 0123456789
  /// ...TEST...
  #[test]
  fn text_center_even() {
    let crossterm = Crossterm::new_with_size(10, 10);
    let word = Word::new("TEST", 0.5, 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 3);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// TEST.....
  #[test]
  fn text_left_uneven() {
    let crossterm = Crossterm::new_with_size(9, 9);
    let word = Word::new("TEST", 0., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// .....TEST
  #[test]
  fn text_right_uneven() {
    let crossterm = Crossterm::new_with_size(9, 9);
    let word = Word::new("TEST", 1., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 5);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// ...TEST..
  #[test]
  fn text_center_uneven() {
    let crossterm = Crossterm::new_with_size(9, 9);
    let word = Word::new("TEST", 0.5, 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 3);
    assert_eq!(y, 0);
  }
}