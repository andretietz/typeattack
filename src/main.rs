use crate::arguments::Arguments;
use crate::crossterm::Crossterm;
use crate::typeattack::Typeattack;

mod arguments;
mod typeattack;
mod crossterm;

#[async_std::main]
async fn main() {
  let args = Arguments::get().unwrap();
  let mut typotack = Typeattack::new(&args, Box::new(Crossterm::new()));
  typotack.run().await;
}
