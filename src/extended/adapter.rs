use common::*;
use extended::common::*;

#[derive(Debug)]
pub struct Incoming(u8);

#[derive(Debug)]
pub struct Outgoing(u8);

pub struct Adapter {
  buffer:      VecDeque<u8>,
  outstanding: u64,
  stream:      extended::delayed_series::Producer,
  sink:        extended::delayed_series::Consumer,
}

impl Adapter {
  pub fn new() -> Adapter {
    Adapter {
      outstanding: 0,
      buffer: VecDeque::new(),
      stream: extended::delayed_series::Producer::new(),
      sink:   extended::delayed_series::Consumer::new(),
    }
  }

  fn try_empty_buffer(&mut self, task_handle: &mut TaskHandle) -> Result<ExtendedAsync<()>, Void> {
    while let Some(item) = self.buffer.pop_front() {
      eprintln!("sending to underlying sink: {}", item);
      if let ExtendedAsyncSink::NotReady(item, agreement_to_notify)
        = self.sink.extended_start_send(task_handle, item)?
      {
        self.buffer.push_front(item);

        // ensure that we attempt to complete any pushes we've started
        self.sink.extended_poll_complete(task_handle)?;

        return Ok(ExtendedAsync::NotReady(agreement_to_notify));
      }
      self.outstanding -= 1;
    }

    Ok(ExtendedAsync::Ready(()))
  }
}

impl Drop for Adapter {
  fn drop(&mut self) {
    eprintln!("adapter dropped with {} outstanding", self.outstanding);
  }
}

impl ExtendedStream for Adapter {
  type Item = Incoming;
  type Error = Void;
  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Option<Self::Item>>, Self::Error>
  {
    loop {
      self.try_empty_buffer(task_handle)?;

      match extended_try_ready!(self.stream.extended_poll(task_handle)) {
        Some(byte) => {
          eprintln!("from underlying stream: {}", byte);
          self.outstanding += 1;
          if byte % 2 == 0 {
            self.buffer.push_back(byte);
          } else {
            return Ok(ExtendedAsync::Ready(Some(Incoming(byte))))
          }
        }
        None => return Ok(ExtendedAsync::Ready(None)),
      }
    }
  }
}

impl Stream for Adapter {
  type Item = Incoming;
  type Error = Void;
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    stream_adapter(self)
  }
}

impl ExtendedSink for Adapter {
  type SinkItem = Outgoing;
  type SinkError = Void;

  fn extended_start_send(&mut self, task_handle: &mut TaskHandle, item: Self::SinkItem)
    -> Result<ExtendedAsyncSink<Self::SinkItem>, Self::SinkError>
  {
    self.try_empty_buffer(task_handle)?;
    self.buffer.push_back(item.0);
    Ok(ExtendedAsyncSink::Ready)
  }

  fn extended_poll_complete(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<()>, Self::SinkError>
  {
    extended_try_ready!(self.try_empty_buffer(task_handle));
    debug_assert!(self.buffer.is_empty());
    self.sink.extended_poll_complete(task_handle)
  }
}

impl Sink for Adapter {
  type SinkItem = Outgoing;
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
  fn basic() {
    let adapter = Adapter::new();
    let (sink, stream) = adapter.split();
    stream
      .take(7)
      .inspect(|x| println!("from stream: {:?}", x))
      .map(|x| Outgoing(x.0))
      .forward(sink).wait().unwrap();
  }
}
