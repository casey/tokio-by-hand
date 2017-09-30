use common::*;
use standard::common::*;

#[derive(Debug)]
pub struct Producer {
  next:   delayed::Producer,
}

impl Producer {
  pub fn new() -> Producer {
    Producer{next: delayed::Producer::new()}
  }
}

impl Stream for Producer {
  type Item = u8;
  type Error = Void;
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let next = try_ready!(self.next.poll());
    self.next = delayed::Producer::new();
    Ok(Async::Ready(Some(next)))
  }
}

pub struct Consumer {
  sending: Option<delayed::Consumer>,
}

impl Consumer {
  pub fn new() -> Consumer {
    Consumer{sending: None}
  }
}

impl Sink for Consumer {
  type SinkItem = u8;
  type SinkError = Void;

  fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
    if self.sending.is_some() {
      self.poll_complete()?;
    }

    if self.sending.is_some() {
      Ok(AsyncSink::NotReady(item))
    } else {
      self.sending = Some(delayed::Consumer::new(item));
      Ok(AsyncSink::Ready)
    }
  }

  fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
    if self.sending.is_some() {
      try_ready!(self.sending.as_mut().unwrap().poll());
      self.sending = None;
    }
    Ok(Async::Ready(()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn values_take_one_second_to_produce() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = Producer::new().take(5);
    let consumer = instant_series::Consumer::new();
    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(5, 500_000_000));
    assert!(elapsed > Duration::new(4, 500_000_000));
  }

  #[test]
  fn values_take_one_second_to_consume() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = instant_series::Producer::new().take(5);
    let consumer = Consumer::new();

    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(5, 500_000_000));
    assert!(elapsed > Duration::new(4, 500_000_000));
  }


  #[test]
  fn production_and_consumption_are_concurrent() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = Producer::new().take(5);
    let consumer = Consumer::new();

    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(6, 500_000_000));
    assert!(elapsed > Duration::new(5, 500_000_000));
  }
}
