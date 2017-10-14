use common::*;

/// A future which resolves after a given duration
pub struct Sleeper {
  until: Instant,
}

impl Sleeper {
  pub fn new(duration: Duration) -> Sleeper {
    Sleeper{until: Instant::now() + duration}
  }
}

impl Future for Sleeper {
  type Item = ();
  type Error = Void;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    let now = Instant::now();
    if now >= self.until {
      Ok(Async::Ready(()))
    } else {
      let remaining = self.until - now;
      let task = task::current();
      thread::spawn(move || {
        thread::sleep(remaining);
        task.notify();
      });
      Ok(Async::NotReady)
    }
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
    assert!(elapsed < Duration::new(1, 300_000_000));
    assert!(elapsed > Duration::new(0, 700_000_000));
  }

  #[test]
  fn with_wait() {
    let start = Instant::now();
    let sleeper = Sleeper::new(Duration::new(1, 0));
    sleeper.wait().unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(1, 300_000_000));
    assert!(elapsed > Duration::new(0, 700_000_000));
  }
}
