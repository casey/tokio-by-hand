use common::*;
use extended::common::*;

const BUFFER_CAPACITY: usize = 10;

/// A sink that consumes one item every second, but which can buffer up to
/// BUFFER_SIZE items
pub struct Consumer {
  buffer: VecDeque<u8>,
  inner:  extended::delayed_series::Consumer,
}

impl Consumer {
  pub fn new() -> Consumer {
    Consumer{buffer: VecDeque::with_capacity(BUFFER_CAPACITY + 1), inner: extended::delayed_series::Consumer::new()}
  }

  fn try_empty_buffer(&mut self, task_handle: &mut TaskHandle) -> Result<ExtendedAsync<()>, Void> {
    while let Some(item) = self.buffer.pop_front() {
      if let ExtendedAsyncSink::NotReady(item, agreement_to_notify)
        = self.inner.extended_start_send(task_handle, item)?
      {
        self.buffer.push_front(item);

        // ensure that we attempt to complete any pushes we've started
        self.inner.extended_poll_complete(task_handle)?;

        return Ok(ExtendedAsync::NotReady(agreement_to_notify));
      }
    }

    Ok(ExtendedAsync::Ready(()))
  }
}

impl ExtendedSink for Consumer {
  type SinkItem = u8;
  type SinkError = Void;

  fn extended_start_send(&mut self, task_handle: &mut TaskHandle, item: Self::SinkItem)
    -> Result<ExtendedAsyncSink<Self::SinkItem>, Self::SinkError>
  {
    if let ExtendedAsync::NotReady(agreement_to_notify) = self.try_empty_buffer(task_handle)? {
      if self.buffer.len() < BUFFER_CAPACITY {
        self.buffer.push_back(item);
        Ok(ExtendedAsyncSink::Ready)
      } else {
        Ok(ExtendedAsyncSink::NotReady(item, agreement_to_notify))
      }
    } else {
      assert!(self.buffer.len() < BUFFER_CAPACITY);
      self.buffer.push_back(item);
      Ok(ExtendedAsyncSink::Ready)
    }
  }

  fn extended_poll_complete(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<()>, Self::SinkError>
  {
    extended_try_ready!(self.try_empty_buffer(task_handle));
    debug_assert!(self.buffer.is_empty());
    self.inner.extended_poll_complete(task_handle)
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
  fn production_and_consumption_are_concurrent() {
    let mut core = Core::new().unwrap();
    let start = Instant::now();
    let producer = extended::delayed_series::Producer::new().take(5);
    let consumer = Consumer::new();

    core.run(producer.forward(consumer)).unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::new(6, 500_000_000));
    assert!(elapsed > Duration::new(5, 500_000_000));
  }
}
