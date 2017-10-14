use common::*;
use extended::common::*;

/// A Stream that produces a random `u8` every second
#[derive(Debug)]
pub struct Producer {
  next: extended::delayed::Producer,
}

impl Producer {
  pub fn new() -> Producer {
    Producer{next: extended::delayed::Producer::new()}
  }
}

impl ExtendedStream for Producer {
  type Item = u8;
  type Error = Void;
  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Option<Self::Item>>, Self::Error>
  {
    let next = extended_try_ready!(self.next.extended_poll(task_handle));
    self.next = extended::delayed::Producer::new();
    Ok(ExtendedAsync::Ready(Some(next)))
  }
}

impl Stream for Producer {
  type Item = u8;
  type Error = Void;
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    stream_adapter(self)
  }
}

/// A Stream that consumes a `u8` every second
pub struct Consumer {
  sending: Option<extended::delayed::Consumer>,
}

impl Consumer {
  pub fn new() -> Consumer {
    Consumer{sending: None}
  }
}

impl ExtendedSink for Consumer {
  type SinkItem = u8;
  type SinkError = Void;

  fn extended_start_send(&mut self, task_handle: &mut TaskHandle, item: Self::SinkItem)
    -> Result<ExtendedAsyncSink<Self::SinkItem>, Self::SinkError>
  {
    if let ExtendedAsync::NotReady(agreement_to_notify) = self.extended_poll_complete(task_handle)? {
      return Ok(ExtendedAsyncSink::NotReady(item, agreement_to_notify))
    }

    assert!(self.sending.is_none());
    self.sending = Some(extended::delayed::Consumer::new(item));

    Ok(ExtendedAsyncSink::Ready)
  }

  fn extended_poll_complete(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<()>, Self::SinkError>
  {
    if self.sending.is_some() {
      extended_try_ready!(self.sending.as_mut().unwrap().extended_poll(task_handle));
      self.sending = None;
    }
    Ok(ExtendedAsync::Ready(()))
  }
}

impl Sink for Consumer {
  type SinkItem = u8;
  type SinkError = Void;

  fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
    sink_start_send_adapter(self, item)
  }

  fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
    sink_poll_complete_adapter(self)
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
    let consumer = extended::instant_series::Consumer::new();
    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(5, 500_000_000));
    assert!(elapsed > Duration::new(4, 500_000_000));
  }

  #[test]
  fn values_take_one_second_to_consume() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = extended::instant_series::Producer::new().take(5);
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
