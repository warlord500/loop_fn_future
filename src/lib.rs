

//! Definition of the `LoopFn` combinator, implementing `Future` loops.

use std::future::Future;
use std::pin::Pin;
use std::task::{Poll,Context};


/// An enum describing whether to `break` or `continue` a `loop_fn` loop.
#[derive(Debug)]
pub enum Loop<T, S> {
    /// Indicates that the loop has completed with output `T`.
    Break(T),

    /// Indicates that the loop function should be called again with input
    /// state `S`.
    Continue(S),
}

/// A future implementing a tail-recursive loop.
///
/// Created by the `loop_fn` function.
#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct LoopFn<A, F> where A: Future {
    future: A,
    func: F,
}


// this is the temporary remplacement until 
// std::task::ready is stablizied.
macro_rules! pend {
	($e:expr) => {
    match $e {
        std::task::Poll::Ready(t) => t,
        std::task::Poll::Pending => {
            return std::task::Poll::Pending;
        }
    }}
}

/// Creates a new future implementing a tail-recursive loop.
///
/// The loop function is immediately called with `initial_state` and should
/// return a value that can be converted to a future. On successful completion,
/// this future should output a `Loop<T, S>` to indicate the status of the
/// loop.
///
/// `Loop::Break(T)` halts the loop and completes the future with output `T`.
///
/// `Loop::Continue(S)` reinvokes the loop function with state `S`. The returned
/// future will be subsequently polled for a new `Loop<T, S>` value.
///
/// # Examples
///
/// ```
/// # extern crate futures;
/// use futures::prelude::*;
/// use futures::future::{self, ok, loop_fn, Loop, FutureResult};
/// use futures::never::Never;
///
/// struct Client {
///     ping_count: u8,
/// }
///
/// impl Client {
///     fn new() -> Self {
///         Client { ping_count: 0 }
///     }
///
///     fn send_ping(self) -> FutureResult<Self, Never> {
///         ok(Client { ping_count: self.ping_count + 1 })
///     }
///
///     fn receive_pong(self) -> FutureResult<(Self, bool), Never> {
///         let done = self.ping_count >= 5;
///         ok((self, done))
///     }
/// }
///
/// # fn main() {
/// let ping_til_done = loop_fn(Client::new(), |client| {
///     client.send_ping()
///         .and_then(|client| client.receive_pong())
///         .and_then(|(client, done)| {
///             if done {
///                 Ok(Loop::Break(client))
///             } else {
///                 Ok(Loop::Continue(client))
///             }
///         })
/// });
/// # }
/// ```
/// small note in the future this code should change A : intoFuture rather than 
/// just future however because it isnt stable on std just use future now!
pub fn loop_fn<S, T, A, F>(initial_state: S, mut func: F) -> LoopFn<A, F>
    where F: FnMut(S) -> A,
          A: Future<Output = Loop<T, S>>,
{
    LoopFn {
        future: func(initial_state),
        func: func,
    }
}

impl<S, T, A :Future, F> Future for LoopFn<A, F>
    where F: FnMut(S) -> A,
          A: Future<Output = Loop<T, S>>,
{
    type Output = T;

    fn poll(self : Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		
		let mut_self  = unsafe {self.get_unchecked_mut()};
		
        loop {
			
		let pinned_future = unsafe { Pin::new_unchecked(&mut mutSelf.future) };
		
            match pend!(pinned_future.poll(cx)) {
                
				Loop::Break(x) => break Poll::Ready(x)
                Loop::Continue(s) => {
					//SAFETY: this is safe because future is dropped before assigment.
					//as well  as no other &mut because by definition can only have one!
					mutSelf.future = (mut_self.func)(s);
				},
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
