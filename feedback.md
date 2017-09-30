"Productive: Tokio makes it easy to implement protocols and program asynchronously."

I think it's important to somehow qualify this statement. I think that it's objectively easier to program asynchronously with node.js or go. Also, using and learning tokio is quite hard. By saying that it's easy, I think we're setting up new users of tokio for frustration, confusion, and feelings of inadequacy.

Also, some people don't understand the difference between async and parallel programming. Newcomers should be aware that if they don't need the performance, thread-based with locking is much easier.

---

"This function will return a future that will resolve to the actual timeout object. The timeout object itself is then a future which will be set to fire at the specified point in the future."

Is this correct?

---

Seperate all io and complex types from discussion. Simple futures, streams, and sinks based on primitive types and timers are all that's necessary to understand tokio. Give end to end examples. Simple futures, streams, and sinks should be provided to understand and play with combinators.

IO is an additional wrinkle that can be added later.

---

Calling `wait()` on a tokio timeout will never complete, since it needs to run in its core. (I think, please correct me if I'm wrong.).

---

One mistake that I made several times was thinking that I had done enough to have things progress to completion, but had missed some step and nothing would happen. This would happen when, for example, I was missing polling a future that I needed to to produce the end-to-end chain of futures that would be driven to completion.

This often happened in implementing sinks. I would poll the underlying consumer future, it would finish, and I would create a new underlying consumer future, but neglect to poll it.

I think the difficulty here is that the graph of futures is implicit the calls to poll(), and it's easy to forget the calls to poll in the places they're needed.

I wonder if there is a lint for this or some kind of diagnostic help. Like if a future is polled and returns not ready, but doesn't poll any other futures.

---

A few times I called `wait` on futures that internally polled timeouts, but were not running on the core whence the timeout came. Could this be caught?

---

Are sink.send_all(stream) and stream.forward(sink) roughly equivalent or exactly equivalent? It wasn't clear from the documentation when I would prefer one or another.

---

In some cases the language in the documentation isn't clear about who's doing what. I think it would be useful to separate the description of each function (`poll`, `start_send`, `flush`, `poll_complete`) into descriptions for the caller (when to call, when not to call, what it will do, what it will return) and the implementor (what it must and must not do, i.e. what contract it must fulfill).

---

My personal opinion is that a thread-local task that can be retrieved with `task::current()` is too magical and confusing. I would prefer that it be passed to every `poll()` function. If this is inefficient, then it could be an uninhabited type that allows the task to be retrieved from thread-local storage, and thus can be optimized out.

I think that `io::Read` and `io::Write` should not be repurposed for non-blocking io. They just feel like very different functions, and the amount of programmer confusion is going to be extreme. The blocking and non-blocking worlds are very different.

---

There is a very uneven difficulty curve.

Simple things that can be done in one place with combinators are relatively easy.

However, if the same needs to be spread between multiple files/functions/structs for clarity or organization, suddenly one needs needs to pass around types which are the result of combinators, which are unweildy and hard to figure out.

Also, if one needs to go beyond the standard combinators, correctly implementing new combinators is extremely difficult.

---

While impl trait and await/async will both help tremendously with making things easier at the higher level, the low-level comprehensibility of the system is still very important. Even if high level tools and shortcuts exist, often times one's understanding is best aided by having a model for how things work at a lower level. This is why I think that it is worth devoting significant energy towards making implementation and use of futures simple and understandable without any syntactic sugar is important and necessary.

---

Why is `self.inner.poll_complete()` needed in `Buffered::try_empty_buffer`.

---

Each future must either:

a. retrieve current task, so that it can unpark it later (may be indirect)
b. complete

If it does neither, than the system will be stuck and won't make progress.

I think that this is an easy mistake to make when implementing futures. I did it several times, and it was frustrating and mysterious. if there was a crash with a good error message, it would have been extremely helpful.

I have no problem with thread local storage, or global variables, or anything like that. However, the current API is very unclear and hard to learn and understand.

If the issue is compatibility with existing Read and Write implementations, we can pass a handle around to Poll, Sink, and Stream, but still have a way for a Read or Write implementation to get the current task from thread local storage.

I think that explicit task is a good idea. It can be purely a type-level construct, a zero-sized type with a method which retrieves the current task from thread local storage. It can also support a method with meaning like "I'm done" or "i have passed the task handle somewhere, so that I can unpark. The return type of poll can include a zero-sized type that facilitates this.

For supporting Read and Write, we can use some mechanism for Read and write to indicate that it has the task handle, so that it can unpark later if necessary. This can then be pulled out of the aether and returned by the future as usual.
