use common::*;
use extended::common::*;

#[derive(Debug)]
pub struct Producer {
  _private: (),
}

impl Producer {
  pub fn new() -> Producer {
    Producer{_private: ()}
  }
}

impl ExtendedFuture for Producer {
  type Item = u8;
  type Error = Void;

  fn extended_poll(&mut self, _task_handle: &mut TaskHandle) -> ExtendedPoll<Self::Item, Self::Error> {
    Ok(ExtendedAsync::Ready(random()))
  }
}

impl Future for Producer {
  type Item = u8;
  type Error = Void;
  fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
    future_adapter(self)
  }
}

pub struct Consumer {
  _private: (),
}

impl Consumer {
  pub fn new(_value: u8) -> Consumer {
    Consumer{_private: ()}
  }
}

impl ExtendedFuture for Consumer {
  type Item = ();
  type Error = Void;
  fn extended_poll(&mut self, _task_handle: &mut TaskHandle) -> ExtendedPoll<Self::Item, Self::Error> {
    Ok(ExtendedAsync::Ready(()))
  }
}

impl Future for Consumer {
  type Item = ();
  type Error = Void;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    future_adapter(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn producer_completes_quickly() {
    let start = Instant::now();
    Producer::new().wait().unwrap();
    assert!(start.elapsed() < Duration::new(0, 200_000_000));
  }

  #[test]
  fn producer_returns_all_values() {
    let mut seen = HashSet::new();
    let expected = 2usize.pow(8);

    while seen.len() < expected {
      seen.insert(Producer::new().wait().unwrap());
    }
  }

  #[test]
  fn consumer_completes_quickly() {
    let start = Instant::now();
    Consumer::new(0).wait().unwrap();
    assert!(start.elapsed() < Duration::new(0, 200_0000_000));
  }
}
