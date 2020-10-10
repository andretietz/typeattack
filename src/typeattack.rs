use std::alloc::System;
use std::borrow::Borrow;
use std::time::{Duration, Instant, SystemTime};

use async_std::process::Output;
use async_std::stream::{interval, Interval};
use futures::{select, Stream, stream::select, StreamExt, TryStreamExt};
use rand::{random, Rng};
use rand::prelude::ThreadRng;

use crate::arguments::Arguments;
use crate::graphics::{Crossterm, RenderEngine};

pub struct State;

pub struct Typeattack<T: RenderEngine> {
  state: State,
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
      state: State,
      level: args.level,
      // TODO: not ideal, fine for now
      words: args.file.clone(),
      engine: Crossterm::new(),
      random: rand::thread_rng(),
    };
  }


  pub async fn run(self: &mut Self) {
    let mut end_condition = true;
    self.engine.clear_screen();
    let mut timer = async_std::stream::interval(Duration::from_millis(100));
    let mut input: &dyn Stream<Item=Event> = self.engine.event_stream().borrow();

    let mut time = SystemTime::now();

    let mut world_state = WorldState::new();

    // TODO unstable method!
    let mut stream = select(
      timer.map(|a| world_state),
      input.map(|a| 2), // TODO: map to game state object
    );

    while let Some(item) = stream.next().await {
      let delta = time.elapsed().unwrap();
      time = SystemTime::now();
      let new_world_state = self.update_world(delta, &world_state);
      self.engine.update(&new_world_state, &world_state);
      world_state = new_world_state;
      // println!("{:?}", world_state);
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
