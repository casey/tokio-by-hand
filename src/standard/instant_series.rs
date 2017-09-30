use common::*;
use standard::common::*;

#[derive(Debug)]
pub struct Producer {
  next: instant::Producer,
}

impl Producer {
  pub fn new() -> Producer {
    Producer{next: instant::Producer::new()}
  }
}

impl Stream for Producer {
  type Item = u8;
  type Error = Void;
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    let next = try_ready!(self.next.poll());
    self.next = instant::Producer::new();
    Ok(Async::Ready(Some(next)))
  }
}

pub struct Consumer {
  sending: Option<instant::Consumer>,
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
      self.sending = Some(instant::Consumer::new(item));
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
