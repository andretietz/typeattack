use crate::crossterm::Crossterm;
use crate::typeattack::Typeattack;

mod typeattack;
mod crossterm;
mod words;

fn main() {
  let mut typotack = Typeattack::new(Box::new(Crossterm::new()));
  typotack.start();
}
