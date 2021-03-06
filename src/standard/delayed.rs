use common::*;
use standard;

/// A Future that produces a random `u8` after a delay of 1 second
pub struct Producer {
  inner:   standard::instant::Producer,
  sleeper: standard::sleeper::Sleeper,
}

impl Producer {
  pub fn new() -> Producer {
    Producer {
      inner:   standard::instant::Producer::new(),
      sleeper: standard::sleeper::Sleeper::new(Duration::new(1, 0)),
    }
  }
}

impl Future for Producer {
  type Item = u8;
  type Error = Void;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    try_ready!(self.sleeper.poll());
    self.inner.poll()
  }
}

impl fmt::Debug for Producer {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.debug_struct("Producer")
      .field("inner", &self.inner)
      .field("timeout", &"...")
      .finish()
  }
}

/// A Future that consumes a `u8` after a delay of 1 second
pub struct Consumer {
  inner:   standard::instant::Consumer,
  sleeper: standard::sleeper::Sleeper,
}

impl Consumer {
  pub fn new(value: u8) -> Consumer {
    Consumer {
      inner:   standard::instant::Consumer::new(value),
      sleeper: standard::sleeper::Sleeper::new(Duration::new(1, 0)),
    }
  }
}

impl Future for Consumer {
  type Item = ();
  type Error = Void;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    try_ready!(self.sleeper.poll());
    self.inner.poll()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn producer_completes_in_about_one_second() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = Producer::new();
    core.run(producer).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(1, 200_000_000));
    assert!(elapsed > Duration::new(0, 800_000_000));
  }

  #[test]
  fn producer_returns_all_values() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let seen = Arc::new(RefCell::new(HashSet::new()));
    let expected = 2usize.pow(8);
    let outstanding = Arc::new(RefCell::new(0u64));

    loop {
      if seen.borrow().len() == expected {
        break;
      }

      while *outstanding.borrow() < 1000 {
        let producer = {
          let seen = seen.clone();
          let outstanding = outstanding.clone();
          Producer::new()
            .map(move |value| { *outstanding.borrow_mut() -= 1; seen.borrow_mut().insert(value); () })
            .map_err(|err| panic!("got error: {}", err))
        };
        *outstanding.borrow_mut() += 1;
        handle.spawn(producer);
      }

      core.turn(Some(Duration::new(0, 100_000_000)));
    }
  }

  #[test]
  fn consumer_completes_in_about_one_second() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let consumer = Consumer::new(0);
    core.run(consumer).unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(1, 200_000_000));
    assert!(elapsed > Duration::new(0, 800_000_000));
  }
}
