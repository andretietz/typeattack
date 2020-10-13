use std::time::{Duration, SystemTime};

use async_std::stream::interval;
use futures::{stream::select, StreamExt};
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::arguments::Arguments;
use crate::graphics::{Crossterm, RenderEngine};

pub struct Typeattack<T: RenderEngine> {
  level: u8,
  words: Vec<String>,
  engine: T,
  random: ThreadRng,
}

pub enum Event {
  Pause,
  Resize(u16, u16),
  AddChar(char),
  RemoveChar,
}

impl Typeattack<Crossterm> {
  pub fn new(args: &Arguments) -> Self {
    return Typeattack {
      level: args.level,
      // TODO: not ideal, fine for now
      words: args.file.clone(),
      engine: Crossterm::new(),
      random: rand::thread_rng(),
    };
  }


  pub async fn run(self: &mut Self) {
    self.engine.clear_screen();
    let timer = interval(Duration::from_millis(100))
        .map(|_| StreamEvent::TimeUpdate);
    let input = self.engine.event_stream()
        .map(|a| StreamEvent::KeyEvent(a));
    // TODO better way for time diff!
    let mut time = SystemTime::now();

    let mut world_state = WorldState::new();

    // unstable method: select
    let mut stream = select(timer, input);

    while let Some(event) = stream.next().await {
      match event {
        StreamEvent::TimeUpdate => {
          let delta = time.elapsed().unwrap();
          time = SystemTime::now();
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
  // between 0..1
  pub x: f64,
  pub y: f64,
}
