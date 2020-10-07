use crate::arguments::Arguments;
use crate::typeattack::Typeattack;

mod arguments;
mod typeattack;
mod graphics;

fn main() -> Result<(), String> {
  let args = Arguments::get()?;
  let typotack = Typeattack::new(&args);
  typotack.run();
  Ok(())
}
