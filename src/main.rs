use crate::arguments::Arguments;
use crate::typotack::Typotack;

mod arguments;
mod typotack;
mod graphics;

fn main() -> Result<(), String> {
  let args = Arguments::get()?;
  let typotack = Typotack::new(&args);
  typotack.run();
  Ok(())
}
