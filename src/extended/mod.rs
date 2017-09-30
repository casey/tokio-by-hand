use common::*;

pub mod common {
  pub use extended::{
    self,
    AgreementToNotify,
    ExtendedAsync,
    ExtendedAsyncSink,
    ExtendedFuture,
    ExtendedSink,
    ExtendedStream,
    ExtendedPoll,
    TaskHandle,
    future_adapter,
    stream_adapter,
    sink_poll_complete_adapter,
    sink_start_send_adapter,
  };
}

pub mod sleeper;
pub mod instant;
pub mod delayed;
pub mod instant_series;
pub mod delayed_series;
pub mod buffered;
pub mod adapter;

#[must_use = "someone needs to call will_notify"]
pub struct TaskHandle {
  _private: (),
}

impl TaskHandle {
  pub fn i_will_notify(&mut self) -> (Task, AgreementToNotify) {
    (task::current(), AgreementToNotify{_private: ()} )
  }
}

#[must_use = "must pass back up chain of futures"]
pub struct AgreementToNotify {
  _private: (),
}

pub enum ExtendedAsync<T> {
  Ready(T),
  NotReady(AgreementToNotify),
}

pub enum ExtendedAsyncSink<T> {
  NotReady(T, AgreementToNotify),
  Ready,
}

pub trait ExtendedFuture {
  type Item;
  type Error;

  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Self::Item>, Self::Error>;
}

pub trait ExtendedSink {
  type SinkItem;
  type SinkError;

  fn extended_start_send(&mut self, task_handle: &mut TaskHandle, item: Self::SinkItem)
    -> Result<ExtendedAsyncSink<Self::SinkItem>, Self::SinkError>;

  fn extended_poll_complete(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<()>, Self::SinkError>;
}

pub trait ExtendedStream {
  type Item;
  type Error;
  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Option<Self::Item>>, Self::Error>;
}


pub fn future_adapter<T, I, E>(extended_future: &mut T) -> Result<Async<I>, E>
  where T: ExtendedFuture<Item=I, Error=E>
{
    match extended_future.extended_poll(&mut TaskHandle{_private: ()}) {
      Ok(ExtendedAsync::Ready(t)) => Ok(Async::Ready(t)),
      Ok(ExtendedAsync::NotReady(_)) => Ok(Async::NotReady),
      Err(err) => Err(err),
    }
}

pub fn stream_adapter<T, I, E>(extended_stream: &mut T) -> Result<Async<Option<I>>, E>
  where T: ExtendedStream<Item=I, Error=E>
{
    match extended_stream.extended_poll(&mut TaskHandle{_private: ()}) {
      Ok(ExtendedAsync::Ready(Some(t))) => Ok(Async::Ready(Some(t))),
      Ok(ExtendedAsync::Ready(None)) => Ok(Async::Ready(None)),
      Ok(ExtendedAsync::NotReady(_)) => Ok(Async::NotReady),
      Err(err) => Err(err),
    }
}

pub fn sink_poll_complete_adapter<T, E>(extended_sink: &mut T) -> Result<Async<()>, E>
  where T: ExtendedSink<SinkError=E>
{
    match extended_sink.extended_poll_complete(&mut TaskHandle{_private: ()}) {
      Ok(ExtendedAsync::Ready(())) => Ok(Async::Ready(())),
      Ok(ExtendedAsync::NotReady(_)) => Ok(Async::NotReady),
      Err(err) => Err(err),
    }
}

pub fn sink_start_send_adapter<T, I, E>(extended_sink: &mut T, item: I) -> Result<AsyncSink<I>, E>
  where T: ExtendedSink<SinkItem=I, SinkError=E>
{
    match extended_sink.extended_start_send(&mut TaskHandle{_private: ()}, item) {
      Ok(ExtendedAsyncSink::Ready) => Ok(AsyncSink::Ready),
      Ok(ExtendedAsyncSink::NotReady(item, _agreement_to_notify)) => Ok(AsyncSink::NotReady(item)),
      Err(err) => Err(err),
    }
}


pub type ExtendedPoll<Item, Error> = Result<ExtendedAsync<Item>, Error>;
