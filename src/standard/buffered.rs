use common::*;
use standard;

const BUFFER_SIZE: usize = 10;

/// A sink that consumes one item every second, but which can buffer up to
/// BUFFER_SIZE items
pub struct Consumer {
  buffer: VecDeque<u8>,
  inner:  standard::delayed_series::Consumer,
}

impl Consumer {
  pub fn new() -> Consumer {
    Consumer {
      buffer: VecDeque::with_capacity(BUFFER_SIZE),
      inner:  standard::delayed_series::Consumer::new(),
    }
  }

  fn try_buffer(&mut self, item: u8) -> AsyncSink<u8> {
    if self.buffer.len() < BUFFER_SIZE {
      self.buffer.push_back(item);
      AsyncSink::Ready
    } else {
      AsyncSink::NotReady(item)
    }
  }
}

impl Sink for Consumer {
  type SinkItem = u8;
  type SinkError = Void;

  fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
    match self.inner.start_send(item)? {
      AsyncSink::NotReady(item) => Ok(self.try_buffer(item)),
      AsyncSink::Ready => Ok(AsyncSink::Ready),
    }
  }

  fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
    try_ready!(self.inner.poll_complete());

    while let Some(item) = self.buffer.pop_front() {
      if let AsyncSink::NotReady(item) = self.inner.start_send(item)? {
        self.buffer.push_front(item);
        break;
      }
      try_ready!(self.inner.poll_complete());
    }
    Ok(Async::Ready(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn production_and_consumption_are_concurrent() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = standard::delayed_series::Producer::new().take(5);
    let consumer = Consumer::new();

    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(6, 500_000_000));
    assert!(elapsed > Duration::new(5, 500_000_000));
  }
}
