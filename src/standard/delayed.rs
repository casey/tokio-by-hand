use common::*;
use standard::common::*;

pub struct Producer {
  inner:   instant::Producer,
  sleeper: sleeper::Sleeper,
}

impl Producer {
  pub fn new() -> Producer {
    Producer {
      inner:   instant::Producer::new(),
      sleeper: sleeper::Sleeper::new(Duration::new(1, 0)),
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

pub struct Consumer {
  inner:   instant::Consumer,
  sleeper: sleeper::Sleeper,
}

impl Consumer {
  pub fn new(value: u8) -> Consumer {
    Consumer {
      inner:   instant::Consumer::new(value),
      sleeper: sleeper::Sleeper::new(Duration::new(1, 0)),
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
