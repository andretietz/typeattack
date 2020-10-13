use std::time::{Duration, SystemTime};

use futures::{stream::select, StreamExt};
use rand::Rng;
use rand::prelude::ThreadRng;

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
  Key(char),
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
    let timer = async_std::stream::interval(Duration::from_millis(100));
    let input = self.engine.event_stream();
    let mut time = SystemTime::now();

    let mut world_state = WorldState::new();

    // unstable method!
    let mut stream = select(
      timer.map(|_| StreamEvent::TimeUpdate),
      input.map(|a| StreamEvent::KeyEvent(a)),
    );

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
          match key {
            Event::Pause => break,
            _ => {}
          }
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

#[derive(Debug)]
pub struct WorldState {
  pub words: Vec<Word>,
  pub buffer: String,
  pub fails: u16,
}

impl WorldState {
  pub fn new() -> Self {
    WorldState {
      words: vec![],
      buffer: String::new(),
      fails: 0,
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
