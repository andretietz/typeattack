use std::pin::Pin;
use std::time::{Duration, Instant};

use async_std::stream::interval;
use futures::{stream::select, StreamExt};
use futures::stream::Stream;
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::arguments::Arguments;

/// Events the [RenderEngine.event_stream] needs to produce.
pub enum Event {
  // pauses the game
  Pause,
  // resizes the screen -> TODO removable
  Resize(u16, u16),
  // a character was entered by the user
  AddChar(char),
  // the user wants to remove the last entered character (delete)
  RemoveChar,
}

pub trait RenderEngine {
  /// Clear the screen.
  fn clear_screen(self: &Self);

  /// TODO can be removed
  fn set_screen_size(self: &mut Self, x: u16, y: u16);

  /// some stream of type Event
  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>>;

  /// when the game has an update, this method is
  /// called in order to update the ui.
  fn update(self: &Self, state: &WorldState, old: &WorldState);
}

pub struct Typeattack {
  level: u8,
  words: Vec<String>,
  engine: Box<dyn RenderEngine>,
  random: ThreadRng,
}

impl Typeattack {
  pub fn new(args: &Arguments, engine: Box<dyn RenderEngine>) -> Self {
    return Typeattack {
      level: args.level,
      // TODO: not ideal, fine for now
      words: args.file.clone(),
      engine,
      random: rand::thread_rng(),
    };
  }


  pub async fn run(self: &mut Self) {
    self.engine.clear_screen();
    // A timer that triggers updates of the ui
    let timer = interval(Duration::from_millis(100))
        .map(|_| StreamEvent::TimeUpdate);
    // A stream that delivers the input of the keyboard
    let input = self.engine.event_stream()
        .map(|a| StreamEvent::KeyEvent(a));

    let mut time = Instant::now();

    let mut world_state = WorldState::new();

    // unstable method: select
    // Create a stream that emits time updates and key events at the same time.
    let mut stream = select(timer, input);

    while let Some(event) = stream.next().await {
      match event {
        StreamEvent::TimeUpdate => {
          let delta = time.elapsed();
          time = Instant::now();
          let new_world_state = self.update_world(delta, &world_state);
          self.engine.update(&new_world_state, &world_state);
          world_state = new_world_state;
        }
        StreamEvent::KeyEvent(key) => {
          let mut new_world_state = world_state.clone();
          match key {
            Event::Pause => break,
            Event::Resize(x, y) => {
              self.engine.set_screen_size(x, y);
              self.engine.clear_screen();
            }
            Event::AddChar(c) => {
              new_world_state.buffer.push(c);
              new_world_state.keycount += 1;
            }
            Event::RemoveChar => {
              new_world_state.buffer.pop();
              new_world_state.keycount += 1;
            }
          }
          let buffer = &new_world_state.buffer;
          new_world_state.words.retain(|word| &word.word != buffer);
          let w = world_state.words.len() - new_world_state.words.len();
          if w > 0 {
            new_world_state.buffer.clear();
          }
          new_world_state.wordcount += w as u128;
          self.engine.update(&new_world_state, &world_state);
          world_state = new_world_state;
        }
      }
    }
  }

  fn update_world(self: &mut Self, delta: Duration, world: &WorldState) -> WorldState {
    let speed: f64 = delta.as_secs_f64() * 0.1;
    let mut words: Vec<Word> = Vec::new();
    let mut fails: u16 = 0;
    for i in 0..world.words.len() {
      let word: &Word = &world.words[i];
      if word.y + speed < 1.0 {
        words.push(Word {
          word: word.word.clone(),
          x: word.x,
          y: word.y + speed,
        })
      } else {
        fails += 1;
      }
    }
    while words.len() < self.level.into() {
      words.push(self.spawn_word())
    }
    WorldState {
      words,
      buffer: world.buffer.clone(),
      fails: world.fails + fails,
      wordcount: world.wordcount,
      keycount: world.keycount,
    }
  }

  fn spawn_word(self: &mut Self) -> Word {
    let word = self.words.get(self.random.gen_range(0, self.words.len())).unwrap();
    Word {
      word: word.clone(),
      x: self.random.gen_range(0.0, 1.0),
      y: 0.0,
    }
  }
}

enum StreamEvent {
  TimeUpdate,
  KeyEvent(Event),
}

#[derive(Debug, Clone)]
pub struct WorldState {
  pub words: Vec<Word>,
  pub buffer: String,
  pub fails: u16,
  pub wordcount: u128,
  pub keycount: u128,
}

impl WorldState {
  pub fn new() -> Self {
    WorldState {
      words: vec![],
      buffer: String::new(),
      fails: 0,
      wordcount: 0,
      keycount: 0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Word {
  pub word: String,
  pub x: f64,
  pub y: f64,
}
