use common::*;
use extended::common::*;

pub struct Sleeper {
  until: Instant,
}

impl Sleeper {
  pub fn new(duration: Duration) -> Sleeper {
    Sleeper{until: Instant::now() + duration}
  }
}

impl ExtendedFuture for Sleeper {
  type Item = ();
  type Error = Void;

  fn extended_poll(&mut self, task_handle: &mut TaskHandle) -> ExtendedPoll<Self::Item, Self::Error> {
    let now = Instant::now();
    if now >= self.until {
      Ok(ExtendedAsync::Ready(()))
    } else {
      let remaining = self.until - now;
      let (task, agreement_to_notify) = task_handle.i_will_notify();
      thread::spawn(move || {
        thread::sleep(remaining);
        task.notify();
      });
      Ok(ExtendedAsync::NotReady(agreement_to_notify))
    }
  }
}

impl Future for Sleeper {
  type Item = ();
  type Error = Void;

  fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
    future_adapter(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn with_core() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let sleeper = Sleeper::new(Duration::new(1, 0));
    core.run(sleeper).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(1, 200_000_000));
    assert!(elapsed > Duration::new(0, 800_000_000));
  }

  #[test]
  fn with_wait() {
    let start = Instant::now();
    let sleeper = Sleeper::new(Duration::new(1, 0));
    sleeper.wait().unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(1, 200_000_000));
    assert!(elapsed > Duration::new(0, 800_000_000));
  }
}
