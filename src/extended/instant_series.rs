use common::*;
use extended::common::*;

/// A Stream that produces a stream of random `u8`s with no delay
#[derive(Debug)]
pub struct Producer {
  next: extended::instant::Producer,
}

impl Producer {
  pub fn new() -> Producer {
    Producer{next: extended::instant::Producer::new()}
  }
}

impl ExtendedStream for Producer {
  type Item = u8;
  type Error = Void;
  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Option<Self::Item>>, Self::Error>
  {
    let next = extended_try_ready!(self.next.extended_poll(task_handle));
    self.next = extended::instant::Producer::new();
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

/// A Sink that consumes `u8`s with no delay
pub struct Consumer {
  sending: Option<extended::instant::Consumer>,
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
    self.sending = Some(extended::instant::Consumer::new(item));

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
  fn producer_returns_all_values() {
    let mut seen = HashSet::new();
    let expected = 2usize.pow(8);
    let mut producer = Producer::new().into_future();

    while seen.len() < expected {
      let (value, next) = producer.wait().unwrap();
      seen.insert(value.unwrap());
      producer = next.into_future();
    }
  }

  #[test]
  fn forward_to_consumer() {
    let expected = 10000;
    let mut produced = 0;
    {
      let producer = Producer::new()
        .inspect(|_| produced += 1 )
        .take(expected);
      let consumer = Consumer::new();

      producer.forward(consumer).wait().unwrap();
    }
    assert_eq!(produced, expected);
  }
}
