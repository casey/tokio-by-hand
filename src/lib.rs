#[macro_use]
extern crate futures;
extern crate rand;
extern crate tokio_core;
extern crate void;

/// Common types used all over the place
pub mod common {
  pub use futures::prelude::*;
  pub use futures::task;
  pub use futures::task::Task;
  pub use rand::random;
  pub use std::cell::RefCell;
  pub use std::collections::{HashSet, VecDeque};
  pub use std::sync::Arc;
  pub use std::sync::atomic::{AtomicBool, Ordering};
  pub use std::thread;
  pub use std::time::{Duration, Instant};
  pub use std::{fmt, io};
  pub use tokio_core::reactor::Core;
  pub use void::Void;
}

/// An extended API version of `try_ready!`
macro_rules! extended_try_ready {
  ( $x:expr ) => {
    {
      match $x {
        Ok(ExtendedAsync::Ready(t)) => t,
        Ok(ExtendedAsync::NotReady(agreement_to_notify)) => {
          return Ok(ExtendedAsync::NotReady(agreement_to_notify));
        }
        Err(err) => {
          return Err(err);
        }
      }
    }
  };
}

/// Simple Futures, Sinks, and Streams using the standard futures API
pub mod standard;

/// Simple Futures, Sinks, and Streams using the extended futures API
pub mod extended;

