use crate::arguments::Arguments;
use crate::typeattack::Typeattack;

mod arguments;
mod graphics;
mod typeattack;

#[async_std::main]
async fn main() {
  let args = Arguments::get().unwrap();
  let mut typotack = Typeattack::new(&args);
  typotack.run().await;
}
