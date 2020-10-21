use std::pin::Pin;
use std::time::{Duration, Instant};

use async_std::stream::interval;
use futures::{stream::select, StreamExt};
use futures::executor::block_on;
use futures::stream::Stream;
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::arguments::Arguments;

/// Events the [RenderEngine.event_stream] needs to produce.
pub enum Event {
  // pauses the game
  Pause,
  // a character was entered by the user
  AddChar(char),
  // the user wants to remove the last entered character (delete)
  RemoveChar,
}

pub trait RenderEngine {
  fn init(self: &Self) -> Result<(), String>;

  /// some stream of type Event
  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>>;

  fn draw_menu(self: &Self);

  /// when the game has an update, this method is
  /// called in order to update the ui.
  fn draw_gamestate(self: &Self, state: &WorldState, old: &WorldState);

  fn draw_result(self: &Self, result: &WorldState);

  fn teardown(self: &Self);
}

pub struct Typeattack {
  level: usize,
  words: Vec<String>,
  engine: Box<dyn RenderEngine>,
  random: ThreadRng,
}

impl Typeattack {
  pub fn new(args: &Arguments, engine: Box<dyn RenderEngine>) -> Self {
    return Typeattack {
      level: 1,
      // TODO: not ideal, fine for now
      words: args.file.clone(),
      engine,
      random: rand::thread_rng(),
    };
  }

  pub fn start(self: &mut Self) {
    if let Err(error) = self.engine.init() {
      println!("{}", error);
      return;
    }
    while block_on(self.show_menu()) {
      let result = block_on(self.show_game());
      block_on(self.show_result(&result));
    }
    self.engine.teardown();
  }

  async fn show_menu(self: &Self) -> bool {
    self.engine.draw_menu();
    let mut input = self.engine.event_stream()
        .filter(|event| {
          futures::future::ready(
            match event {
              Event::AddChar(_) => true,
              Event::Pause => true,
              _ => false
            }
          )
        });
    if let Some(event) = input.next().await {
      return match event {
        Event::Pause => false,
        _ => true
      };
    }
    true
  }

  async fn show_game(self: &mut Self) -> WorldState {
    // A timer that triggers updates of the ui 60 FPS ~ 16.666_7ms => 16ms
    let timer = interval(Duration::from_millis(16))
        .map(|_| StreamEvent::TimeUpdate);
    // A stream that delivers the input of the keyboard
    let input = self.engine.event_stream()
        .map(|a| StreamEvent::KeyEvent(a));

    let time = Instant::now();
    let mut last = 0;
    let mut world_state = WorldState::new();

    // unstable method: select
    // Create a stream that emits time updates and key events at the same time.
    let mut stream = select(timer, input);

    while let Some(event) = stream.next().await {
      match event {
        StreamEvent::TimeUpdate => {
          let timestamp = time.elapsed().as_millis();
          let delta = timestamp - last;
          let new_world_state = self.update_world(delta, &world_state);
          self.engine.draw_gamestate(&new_world_state, &world_state);
          world_state = new_world_state;
          last = timestamp;
        }
        StreamEvent::KeyEvent(key) => {
          let mut new_world_state = world_state.clone();
          match key {
            Event::Pause => break,
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
          let removed_words = world_state.words.len() - new_world_state.words.len();
          if removed_words > 0 {
            new_world_state.buffer.clear();
          }
          new_world_state.wordcount += removed_words as u128;
          self.engine.draw_gamestate(&new_world_state, &world_state);
          world_state = new_world_state;
        }
      }
      if world_state.fails >= 3 {
        break;
      }
    }
    world_state
  }

  async fn show_result(self: &Self, result: &WorldState) {
    self.engine.draw_result(result);
    self.engine.event_stream().next().await;
  }

  fn update_world(self: &mut Self, delta: u128, world: &WorldState) -> WorldState {
    let velocity = 0.0001 * (1.0 + self.level as f64 / 10.0) as f64;
    // v = 1.0(screen_unit) / 10000ms = 0.0001 screen_unit/ms
    // delta_s = v * delta_t
    let delta_s = velocity * delta as f64;
    let mut words: Vec<Word> = Vec::new();
    let mut fails: u16 = 0;
    for i in 0..world.words.len() {
      let word: &Word = &world.words[i];
      if word.y + delta_s < 1.0 {
        words.push(Word::new(word.word.as_str().clone(), word.x, word.y + delta_s));
      } else {
        fails += 1;
      }
    }
    self.level = (world.keycount / 10) as usize + 1;
    while words.len() < (self.level / 5) + 1 {
      words.push(self.spawn_word())
    }
    WorldState {
      words,
      buffer: world.buffer.clone(),
      fails: world.fails + fails,
      wordcount: world.wordcount,
      keycount: world.keycount,
      level: world.level,
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
  pub level: u128,
}

impl WorldState {
  pub fn new() -> Self {
    WorldState {
      words: vec![],
      buffer: String::new(),
      fails: 0,
      wordcount: 0,
      keycount: 0,
      level: 1,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Word {
  pub word: String,
  pub x: f64,
  pub y: f64,
}

impl Word {
  pub fn new(value: &str, x: f64, y: f64) -> Self {
    return Self {
      word: String::from(value),
      x,
      y,
    };
  }
}
