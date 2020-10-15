use std::pin::Pin;

use futures::stream::Stream;

use crate::typeattack::{Event, WorldState};

pub trait RenderEngine {
  /// Clear the screen.
  fn clear_screen(self: &Self);

  ///
  fn set_screen_size(self: &mut Self, x: u16, y: u16);

  /// some stream of type Event
  fn event_stream(self: &Self) -> Pin<Box<dyn Stream<Item=Event>>>;

  /// when the game has an update, this method is
  /// called in order to update the ui.
  fn update(self: &Self, state: &WorldState, old: &WorldState);
}
