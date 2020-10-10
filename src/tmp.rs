use std::thread::{sleep, spawn};
use std::time::Duration;

use futures::executor::block_on;


struct smth {
  pub i: i32
}

fn main() {
  println!("First Line: {}", std::thread::current().name().unwrap());

  let thread = std::thread::Builder::new()
      .name("NewThread".to_string()).spawn( || {
    println!("Remote Line: {}", std::thread::current().name().unwrap());

    block_on(futures::future::join3(
      async_std::task::spawn(do_smth(1)),
      async_std::task::spawn(do_smth(2)),
      async_std::task::spawn(do_smth_different())
    ));
  }).unwrap();
  println!("Last Line: {}", std::thread::current().name().unwrap());
  thread.join().unwrap();
}

async fn do_smth_different() {
  println!("I am different...")
}

async fn do_smth(time: u64) -> i32 {
  let mut i = smth { i: 0 };
  dec(&mut i, time).await;
  std::thread::sleep(Duration::new(time, 0));
  inc(&mut i, time).await;
  return i.i;
}

async fn inc(v: &mut smth, id:u64) {
  println!("increment({}): {}", id, std::thread::current().name().unwrap());
  v.i = v.i + 1;
}
async fn dec(v: &mut smth, id:u64) {
  println!("decrement({}): {}", id, std::thread::current().name().unwrap());
  v.i = v.i - 1;
}