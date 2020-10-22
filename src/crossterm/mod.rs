mod strings;

use std::io::{stdout, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use strings::*;

use crossterm::{
  cursor::{Hide, MoveTo, RestorePosition, SavePosition},
  event::{self, KeyCode},
  execute,
  queue,
  style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType, enable_raw_mode, size},
};
use crossterm::cursor::EnableBlinking;
use crossterm::terminal::disable_raw_mode;
use futures::stream::{Stream, StreamExt};

use crate::typeattack::{Event, RenderEngine, Word, WorldState};

struct Screen {
  size_x: u16,
  size_y: u16,
  unit_x: f64,
  unit_y: f64,
}

impl Screen {
  pub fn new(x: u16, y: u16) -> Self {
    Screen {
      size_x: x,
      size_y: y,
      unit_x: 1.0 / x as f64,
      unit_y: 1.0 / y as f64,
    }
  }
}

pub struct Crossterm {
  screen: Arc<Mutex<Screen>>
}

impl Crossterm {
  pub fn new() -> Self {
    let (x, y) = size().unwrap();
    Crossterm::new_with_size(x, y)
  }

  pub fn new_with_size(x: u16, y: u16) -> Self {
    let instance = Crossterm {
      screen: Arc::new(Mutex::new(Screen::new(0, 0)))
    };
    Self::set_screen_size(&instance.screen, x, y);
    instance
  }

  fn print_word(self: &Self, buffer: &String, word: &Word) {
    let (x, y) = self.get_position(word);
    queue!(stdout(),
      MoveTo(x, y),
      Print(&word.word)
    ).unwrap();

    if word.word.starts_with(buffer.as_str()) {
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
    let screen = self.screen.lock().unwrap();
    let word_size = word.word.len() as f64 * screen.unit_x;
    let max = (screen.size_x as f64 * screen.unit_x) - word_size;
    // 1/max = value/x
    // x = max*value/1
    let x = max * (screen.size_x as f64 * word.x) / 1.0;
    let y = (screen.size_y - 1) as f64 * word.y;
    (x.round() as u16, y.round() as u16)
  }

  fn stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>> {
    let screen = self.screen.clone();
    return event::EventStream::new()
        // filter all events we don't need
        .filter(move |result| {
          return futures::future::ready(match result {
            Ok(event) => {
              match event {
                event::Event::Key(key) => {
                  match key.code {
                    KeyCode::Esc => true,
                    KeyCode::Backspace => true,
                    KeyCode::Char(_) => true,
                    KeyCode::Enter => true,
                    _ => false
                  }
                }
                event::Event::Resize(x, y) => {
                  Crossterm::set_screen_size(&screen, *x, *y);
                  false
                }
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
                    KeyCode::Esc => Event::Stop,
                    KeyCode::Backspace => Event::RemoveChar,
                    KeyCode::Char(c) => Event::AddChar(c),
                    KeyCode::Enter => Event::ClearBuffer,
                    _ => panic!("")
                  }
                }
                _ => panic!("")
              }
            }
            _ => panic!("")
          };
        })
        .boxed();
  }

  fn set_screen_size(target: &Arc<Mutex<Screen>>, x: u16, y: u16) {
    if x < 80 || y < 24 {
      panic!("The terminal size needs to be at least 80x24!")
    }
    let mut screen = target.lock().unwrap();
    screen.size_x = x;
    screen.size_y = y - 1;
    screen.unit_x = 1.0 / screen.size_x as f64;
    screen.unit_y = 1.0 / screen.size_y as f64;
  }
}

impl RenderEngine for Crossterm {
  fn init(self: &Self) -> Result<(), String> {
    enable_raw_mode().map_err(|_| "Terminal is not supported!")?;
    execute!(stdout(),
      SetForegroundColor(Color::White),
      SetBackgroundColor(Color::Black),
      Hide,
      SavePosition
    ).unwrap();
    Ok(())
  }

  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>> {
    self.stream()
  }

  fn draw_menu(self: &Self) {
    let screen = self.screen.lock().unwrap();
    let title_pos_y = (screen.size_y - 6) / 3;
    let title_pos_x = (screen.size_x - INTRO1.len() as u16) / 2;
    let help_pos_x = (screen.size_x - HELP.len() as u16) / 2;
    let copyright_pos_x = (screen.size_x - COPYRIGHT.len() as u16) / 2;
    execute!(stdout(),
      Clear(ClearType::All),
      MoveTo(title_pos_x, title_pos_y + 0),
      Print(INTRO1),
      MoveTo(title_pos_x, title_pos_y + 1),
      Print(INTRO2),
      MoveTo(title_pos_x, title_pos_y + 2),
      Print(INTRO3),
      MoveTo(title_pos_x, title_pos_y + 3),
      Print(INTRO4),
      MoveTo(title_pos_x, title_pos_y + 4),
      Print(INTRO5),
      MoveTo(title_pos_x,title_pos_y + 5),
      Print(INTRO6),
      MoveTo(help_pos_x,title_pos_y + 8),
      Print(HELP),
      MoveTo(copyright_pos_x, screen.size_y - 1),
      Print(COPYRIGHT)
    ).unwrap();
  }

  fn draw_gamestate(self: &Self, state: &WorldState, _: &WorldState) {
    queue!(stdout(), Clear(ClearType::All)).unwrap();
    // update new words
    for word in &state.words {
      self.print_word(&state.buffer, &word);
    }

    // draw HUD
    queue!(stdout(),
      MoveTo(0, self.screen.lock().unwrap().size_y),
      Print(format!("Level: {} Fails: {} Words: {} Inputs: {} Buffer: {}",
        &state.level,
        &state.fails,
        &state.wordcount,
        &state.keycount,
        &state.buffer
        ))
      ).unwrap();
    // apply
    stdout().flush().unwrap();
  }

  fn draw_result(self: &Self, _result: &WorldState) {
    let gameover_pos_x = (self.screen.lock().unwrap().size_x - GAME_OVER1.len() as u16) / 2;
    let gameover_pos_y = (self.screen.lock().unwrap().size_y-8) / 2;
    queue!(stdout(),
      Clear(ClearType::All),
      MoveTo(gameover_pos_x, gameover_pos_y + 0),
      Print(GAME_OVER1),
      MoveTo(gameover_pos_x, gameover_pos_y + 1),
      Print(GAME_OVER2),
      MoveTo(gameover_pos_x, gameover_pos_y + 2),
      Print(GAME_OVER3),
      MoveTo(gameover_pos_x, gameover_pos_y + 3),
      Print(GAME_OVER4),
      MoveTo(gameover_pos_x, gameover_pos_y + 4),
      Print(GAME_OVER5),
      MoveTo(gameover_pos_x, gameover_pos_y + 5),
      Print(GAME_OVER6),
      MoveTo(gameover_pos_x, gameover_pos_y + 6),
      Print(GAME_OVER7),
      MoveTo(gameover_pos_x, gameover_pos_y + 7),
      Print(GAME_OVER8)

    ).unwrap();
    // apply
    stdout().flush().unwrap();
  }

  fn teardown(self: &Self) {
    disable_raw_mode().unwrap();
    execute!(stdout(),
      EnableBlinking,
      RestorePosition,
      Clear(ClearType::All),
    ).unwrap();
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
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 0., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
  }

  /// 0123456789
  /// ......TEST
  #[test]
  fn text_right_even() {
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 1., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 76);
    assert_eq!(y, 0);
  }

  /// 0123456789
  /// ...TEST...
  #[test]
  fn text_center_even() {
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 0.5, 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 38);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// TEST.....
  #[test]
  fn text_left_uneven() {
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 0., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 0);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// .....TEST
  #[test]
  fn text_right_uneven() {
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 1., 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 76);
    assert_eq!(y, 0);
  }

  /// 012345678
  /// ...TEST..
  #[test]
  fn text_center_uneven() {
    let crossterm = Crossterm::new_with_size(80, 24);
    let word = Word::new("TEST", 0.5, 0.);
    let (x, y) = crossterm.get_position(&word);
    assert_eq!(x, 38);
    assert_eq!(y, 0);
  }
}