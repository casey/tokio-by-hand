tokio by hand
=============

I've been trying to wrap my head around futures for a while, and it's been a pretty rough ride.

The project that I started off working on was to consume a cryptocurrency exchange market data websocket feed and write it to a database.

I started using https://crates.io/crates/websocket and decided to make it async. Although I could have gotten away with using threads, locks, and channels, I had never used tokio and futures in any depth before, so I used it as an excuse to learn.

My experience was basically that the provided combinators were easy enough to use, but whenever I needed to go beyond them, things got difficult very quickly. The learning curve was very uneven, easy things were manageable, but all of a sudden when things couldn't be done with the basic combinators, I was pretty stuck.

There were a couple reasons that sometimes the combinators weren't enough:

1. Program organization. I had a huge mass of combinator calls that did everything I wanted, but it was all in a single function and was difficult to read and debug. I was reading from the websocket, writing back control messages to the websocket, deserializing text from the websocket into my local structures, and serializing higher-level control messages to text and writing them back into the websocket.

  My thought was that I could abstract deserialization, serialization, and low level management of the websocket into another Sink + Stream over the higher level, deserialized types, but this meant implementing a future myself, which is much more difficult than successfully using combinators.

2. Crazy types. I found that combinators generally produced incomprehensible types. If I implemented my own concrete futures over concrete types, the program would be much much simpler. Boxing sort of worked, but sometimes type inference failed in baffling ways, and I needed a lot of annotations to coax things in the correct types

Trying to implement, as my first future, a very complicated mess of deserialization, serialization, and websocket connection management was, as you can imagine, very difficult.

So, I stepped back and started implementing a bunch of simpler futures, so that I could understand how things worked.

I implemented the following futures:
- `standard::sleeper::Sleeper`, a future that produces `()` after a timeout by sleeping on a spawned thread.
- `standard::instant::{Producer,Consumer}` futures that produce and consume a `u8`. A future that consumes a `u8` is a bit unusual, but I used it as the basis for implementing sinks.
- `standard::delayed::{Producer,Consumer}` same as above, but operate after a timeout
- `standard::instant_series::{Producer,Consumer}` a Stream and a Sink that produce an infinite sequence of random `u8`s
- `standard::instant_series::{Producer,Consumer}` same as above, but with a one second delay after each item
- `standard::buffered::Consumer` same as the above consumer, but contains a fixed size buffer in case items are received more rapidly than they can be processed.

When implementing them, my main difficulties in understanding the futures model were as follows:

1. I found the lack of an explicit task argument to be very confusing. It wasn't obvious to me what a task was, how one was created, what it represented, and how a task could be notified.

  I thought about commenting on https://github.com/alexcrichton/futures-rs/issues/129 , but I felt like it would mostly be a "me too" comment, so I held off.

2. Understanding when a future had "done enough" to ensure that it would make progress in the future. More than a few times I would implement a future and expect it work, but hadn't actually arranged, directly or indirectly, for the task to be notified, so everything would hang.

After figuring out where my difficulties lay, I created a modified futures API that would make it hard to avoid these same mistakes.

You can see the API in the `extended` module. It adds an explicit `TaskHandle` to `poll`, `start_send`, and `poll_complete`. This was to help with #1. To help with #2, I extended the `Async` and `AsyncSink` types to also, when not ready, contain an `AgreementToNotify` value.

An `AgreementToNotify` value is is obtained by calling `TaskHandle::i_will_notify()`, which returns `(Task, AgreementToNotify)`. (Internally, the `Task` is obtained from `task::current()`.) The name of the method is admittadly a bit on-the-nose, but the idea is to mark in your future implementation the point where you're actually getting the current task so that you can notify it in the future.

In practice, I only did this once in the codebase, in the implementation of `extended::sleeper::Sleeper`. However, this was still very useful, since it made clear whether a future was the last in a chain of futures, and was thus responsible for notifying the task, or whether it was merely delegating to another future.

If delegating to another future, it would have to get the `AgreementToNotify` from a `NotReady` from another future, and it was thus impossible to return `NotReady` without directly or indirectly arranging for the task to be notified.

The extended API made things a _lot_ clearer to me. I understand that it's an extremely verbose API, but I think that it aids greatly in understanding the futures model, makes the contracts of the the traits extremely clear, and moves many errors to compile time. At least, I found that to be the case for me.
