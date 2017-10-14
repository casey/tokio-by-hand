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

/// A handle to the current task
pub struct TaskHandle {
  _private: (),
}

impl TaskHandle {
  pub fn i_will_notify(&mut self) -> (Task, AgreementToNotify) {
    (task::current(), AgreementToNotify{_private: ()} )
  }
}

/// An object representing an agreement to notify the current task. 
///
/// To construct an `ExtendedAsync::NotReady` or `ExtendedAsyncSink::NotReady`
/// to return you must be able to provide an `AgreementToNotify`. If such an
/// agreement has been obtained from a correctly implemented Future, Sink, or
/// Stream, then that Future, Sink, or Stream has agreed to notify the current
/// task when progress can be made.
///
/// If you are the last in a chain of futures, you must call
/// `TaskHandle::i_will_notify()` to obtain an `AgreementToNotify`, which you
/// must honor yourself by arranging to notify the Task, mostly likely by passing
/// it to another thread and notifying it when progress can be made.
pub struct AgreementToNotify {
  _private: (),
}

/// The extended API equivalent of `Async`
pub enum ExtendedAsync<T> {
  Ready(T),
  NotReady(AgreementToNotify),
}

/// The extended API equivalent of `AsyncSink`
pub enum ExtendedAsyncSink<T> {
  NotReady(T, AgreementToNotify),
  Ready,
}

/// The extended API equivalent of the `Future` trait
pub trait ExtendedFuture {
  type Item;
  type Error;

  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Self::Item>, Self::Error>;
}

/// The extended API equivalent of the `Sink` trait
pub trait ExtendedSink {
  type SinkItem;
  type SinkError;

  fn extended_start_send(&mut self, task_handle: &mut TaskHandle, item: Self::SinkItem)
    -> Result<ExtendedAsyncSink<Self::SinkItem>, Self::SinkError>;

  fn extended_poll_complete(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<()>, Self::SinkError>;
}

/// The extended API equivalent of the `Stream` trait
pub trait ExtendedStream {
  type Item;
  type Error;
  fn extended_poll(&mut self, task_handle: &mut TaskHandle)
    -> Result<ExtendedAsync<Option<Self::Item>>, Self::Error>;
}

/// An adaptor function to be used when implementing a standard API `Future`
/// with an implementation of an extended API `Future`.
pub fn future_adapter<T, I, E>(extended_future: &mut T) -> Result<Async<I>, E>
  where T: ExtendedFuture<Item=I, Error=E>
{
    match extended_future.extended_poll(&mut TaskHandle{_private: ()}) {
      Ok(ExtendedAsync::Ready(t)) => Ok(Async::Ready(t)),
      Ok(ExtendedAsync::NotReady(_)) => Ok(Async::NotReady),
      Err(err) => Err(err),
    }
}

/// An adaptor function to be used when implementing a standard API `Stream`
/// with an implementation of an extended API `Stream`.
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

/// An adaptor function to be used when implementing a standard API `Sink::poll_complete`
/// with an implementation of an extended API `Sink`.
pub fn sink_poll_complete_adapter<T, E>(extended_sink: &mut T) -> Result<Async<()>, E>
  where T: ExtendedSink<SinkError=E>
{
    match extended_sink.extended_poll_complete(&mut TaskHandle{_private: ()}) {
      Ok(ExtendedAsync::Ready(())) => Ok(Async::Ready(())),
      Ok(ExtendedAsync::NotReady(_)) => Ok(Async::NotReady),
      Err(err) => Err(err),
    }
}

/// An adaptor function to be used when implementing a standard API `Sink::start_send`
/// with an implementation of an extended API `Sink`.
pub fn sink_start_send_adapter<T, I, E>(extended_sink: &mut T, item: I) -> Result<AsyncSink<I>, E>
  where T: ExtendedSink<SinkItem=I, SinkError=E>
{
    match extended_sink.extended_start_send(&mut TaskHandle{_private: ()}, item) {
      Ok(ExtendedAsyncSink::Ready) => Ok(AsyncSink::Ready),
      Ok(ExtendedAsyncSink::NotReady(item, _agreement_to_notify)) => Ok(AsyncSink::NotReady(item)),
      Err(err) => Err(err),
    }
}

/// The extended API version of `Poll`
pub type ExtendedPoll<Item, Error> = Result<ExtendedAsync<Item>, Error>;
