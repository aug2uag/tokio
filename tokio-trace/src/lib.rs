//! A scoped, structured logging and diagnostics system.
//!
//! # Overview
//!
//! `tokio-trace` is a framework for instrumenting Rust programs to collect
//! structured, event-based diagnostic information.
//!
//! In asynchronous systems like Tokio, interpreting traditional log messages can
//! often be quite challenging. Since individual tasks are multiplexed on the same
//! thread, associated events and log lines are intermixed making it difficult to
//! trace the logic flow. `tokio-trace` expands upon logging-style diagnostics by
//! allowing libraries and applications to record structured events with additional
//! information about *temporality* and *causality* --- unlike a log message, a span
//! in `tokio-trace` has a beginning and end time, may be entered and exited by the
//! flow of execution, and may exist within a nested tree of similar spans. In
//! addition, `tokio-trace` spans are *structured*, with the ability to record typed
//! data as well as textual messages.
//!
//! The `tokio-trace` crate provides the APIs necessary for instrumenting libraries
//! and applications to emit trace data.
//!
//! # Core Concepts
//!
//! The core of `tokio-trace`'s API is composed of `Event`s, `Span`s, and
//! `Subscriber`s. We'll cover these in turn.
//!
//! ## `Span`s
//!
//! A [`Span`] represents a _period of time_ during which a program was executing
//! in some context. A thread of execution is said to _enter_ a span when it
//! begins executing in that context, and to _exit_ the span when switching to
//! another context. The span in which a thread is currently executing is
//! referred to as the _current_ span.
//!
//! Spans form a tree structure --- unless it is a root span, all spans have a
//! _parent_, and may have one or more _children_. When a new span is created,
//! the current span becomes the new span's parent. The total execution time of
//! a span consists of the time spent in that span and in the entire subtree
//! represented by its children. Thus, a parent span always lasts for at least
//! as long as the longest-executing span in its subtree.
//!
//! In addition, data may be associated with spans. A span may have _fields_ ---
//! a set of key-value pairs describing the state of the program during that
//! span; an optional name, and metadata describing the source code location
//! where the span was originally entered.
//!
//! ## Events
//!
//! An [`Event`] represents a _point_ in time. It signifies something that
//! happened while the trace was executing. `Event`s are comparable to the log
//! records emitted by unstructured logging code, but unlike a typical log line,
//! an `Event` may occur within the context of a `Span`. Like a `Span`, it
//! may have fields, and implicitly inherits any of the fields present on its
//! parent span, and it may be linked with one or more additional
//! spans that are not its parent; in this case, the event is said to _follow
//! from_ those spans.
//!
//! Essentially, `Event`s exist to bridge the gap between traditional
//! unstructured logging and span-based tracing. Similar to log records, they
//! may be recorded at a number of levels, and can have unstructured,
//! human-readable messages; however, they also carry key-value data and exist
//! within the context of the tree of spans that comprise a trase. Thus,
//! individual log record-like events can be pinpointed not only in time, but
//! in the logical execution flow of the system.
//!
//! Events are represented as a special case of spans --- they are created, they
//! may have fields added, and then they close immediately, without being
//! entered.
//!
//! ## `Subscriber`s
//!
//! As `Span`s and `Event`s occur, they are recorded or aggregated by
//! implementations of the [`Subscriber`] trait. `Subscriber`s are notified
//! when an `Event` takes place and when a `Span` is entered or exited. These
//! notifications are represented by the following `Subscriber` trait methods:
//! + [`observe_event`], called when an `Event` takes place,
//! + [`enter`], called when execution enters a `Span`,
//! + [`exit`], called when execution exits a `Span`
//!
//! In addition, subscribers may implement the [`enabled`] function to _filter_
//! the notifications they receive based on [metadata] describing each `Span`
//! or `Event`. If a call to `Subscriber::enabled` returns `false` for a given
//! set of metadata, that `Subscriber` will *not* be notified about the
//! corresponding `Span` or `Event`. For performance reasons, if no currently
//! active subscribers express  interest in a given set of metadata by returning
//! `true`, then the corresponding `Span` or `Event` will never be constructed.
//!
//! # Usage
//!
//! First, add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-trace = { git = "https://github.com/tokio-rs/tokio" }
//! ```
//!
//! Next, add this to your crate:
//!
//! ```rust
//! #[macro_use]
//! extern crate tokio_trace;
//! # fn main() {}
//! ```
//!
//! `Span`s are constructed using the `span!` macro, and then _entered_
//! to indicate that some code takes place within the context of that `Span`:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate tokio_trace;
//! # fn main() {
//! // Construct a new span named "my span".
//! let mut span = span!("my span");
//! span.enter(|| {
//!     // Any trace events in this closure or code called by it will occur within
//!     // the span.
//! });
//! // Dropping the span will close it, indicating that it has ended.
//! # }
//! ```
//!
//! `Event`s are created using the `event!` macro, and are recorded when the
//! event is dropped:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate tokio_trace;
//! # fn main() {
//! use tokio_trace::Level;
//! event!(Level::INFO, "something has happened!");
//! # }
//! ```
//!
//! Users of the [`log`] crate should note that `tokio-trace` exposes a set of macros for
//! creating `Event`s (`trace!`, `debug!`, `info!`, `warn!`, and `error!`) which may
//! be invoked with the same syntax as the similarly-named macros from the `log`
//! crate. Often, the process of converting a project to use `tokio-trace` can begin
//! with a simple drop-in replacement.
//!
//! Let's consider the `log` crate's yak-shaving example:
//!
//! ```rust
//! #[macro_use]
//! extern crate tokio_trace;
//! use tokio_trace::field;
//! # fn main() {
//! pub fn shave_the_yak(yak: &mut Yak) {
//!     // Create a new span for this invocation of `shave_the_yak`, annotated
//!     // with  the yak being shaved as a *field* on the span.
//!     span!("shave_the_yak", yak = field::debug(yak)).enter(|| {
//!         // Since the span is annotated with the yak, it is part of the context
//!         // for everything happening inside the span. Therefore, we don't need
//!         // to add it to the message for this event, as the `log` crate does.
//!         info!(target: "yak_events", "Commencing yak shaving");
//!
//!         loop {
//!             match find_a_razor() {
//!                 Ok(razor) => {
//!                     // We can add the razor as a field rather than formatting it
//!                     // as part of the message, allowing subscribers to consume it
//!                     // in a more structured manner:
//!                     info!({ razor = field::display(razor) }, "Razor located");
//!                     yak.shave(razor);
//!                     break;
//!                 }
//!                 Err(err) => {
//!                     // However, we can also create events with formatted messages,
//!                     // just as we would for log records.
//!                     warn!("Unable to locate a razor: {}, retrying", err);
//!                 }
//!             }
//!         }
//!     })
//! }
//! # }
//! ```
//!
//! You can find examples showing how to use this crate in the examples directory.
//!
//! ### In libraries
//!
//! Libraries should link only to the `tokio-trace` crate, and use the provided
//! macros to record whatever information will be useful to downstream consumers.
//!
//! ### In executables
//!
//! In order to record trace events, executables have to use a `Subscriber`
//! implementation compatible with `tokio-trace`. A `Subscriber` implements a way of
//! collecting trace data, such as by logging it to standard output.
//!
//! Unlike the `log` crate, `tokio-trace` does *not* use a global `Subscriber` which
//! is initialized once. Instead, it follows the `tokio` pattern of executing code
//! in a context. For example:
//!
//! ```rust
//! #[macro_use]
//! extern crate tokio_trace;
//! # pub struct FooSubscriber;
//! # use tokio_trace::{span::Id, field::Field, Metadata};
//! # impl tokio_trace::Subscriber for FooSubscriber {
//! #   fn new_span(&self, _: &Metadata) -> Id { Id::from_u64(0) }
//! #   fn record_debug(&self, _: &Id, _: &Field, _: &::std::fmt::Debug) {}
//! #   fn add_follows_from(&self, _: &Id, _: Id) {}
//! #   fn enabled(&self, _: &Metadata) -> bool { false }
//! #   fn enter(&self, _: &Id) {}
//! #   fn exit(&self, _: &Id) {}
//! # }
//! # impl FooSubscriber {
//! #   fn new() -> Self { FooSubscriber }
//! # }
//! # fn main() {
//! let my_subscriber = FooSubscriber::new();
//!
//! tokio_trace::subscriber::with_default(my_subscriber, || {
//!     // Any trace events generated in this closure or by functions it calls
//!     // will be collected by `my_subscriber`.
//! })
//! # }
//! ```
//!
//! This approach allows trace data to be collected by multiple subscribers within
//! different contexts in the program. Alternatively, a single subscriber may be
//! constructed by the `main` function and all subsequent code executed with that
//! subscriber as the default. Any trace events generated outside the context of a
//! subscriber will not be collected.
//!
//! The executable itself may use the `tokio-trace` crate to instrument itself as well.
//!
//! [`log`]: https://docs.rs/log/0.4.6/log/
//! [`Span`]: span/struct.Span
//! [`Event`]: struct.Event.html
//! [`Subscriber`]: subscriber/trait.Subscriber.html
//! [`observe_event`]: subscriber/trait.Subscriber.html#tymethod.observe_event
//! [`enter`]: subscriber/trait.Subscriber.html#tymethod.enter
//! [`exit`]: subscriber/trait.Subscriber.html#tymethod.exit
//! [`enabled`]: subscriber/trait.Subscriber.html#tymethod.enabled
//! [metadata]: struct.Metadata.html
extern crate tokio_trace_core;

// Somehow this `use` statement is necessary for us to re-export the `core`
// macros on Rust 1.26.0. I'm not sure how this makes it work, but it does.
#[allow(unused_imports)]
use tokio_trace_core::*;

pub use self::{
    dispatcher::Dispatch,
    field::Value,
    span::{Event, Id, Span},
    subscriber::Subscriber,
    tokio_trace_core::{
        callsite::{self, Callsite},
        metadata, Level, Metadata,
    },
};

/// Constructs a new static callsite for a span or event.
#[macro_export]
macro_rules! callsite {
    (span: $name:expr, $( $field_name:ident ),*) => ({
        callsite!(@
            name: $name,
            target: module_path!(),
            level: $crate::Level::TRACE,
            fields: &[ $(stringify!($field_name)),* ]
        )
    });
    (event: $lvl:expr, $( $field_name:ident ),*) =>
        (callsite!(event: $lvl, target: module_path!(), $( $field_name ),* ));
    (event: $lvl:expr, target: $target:expr, $( $field_name:ident ),*) => ({
        callsite!(@
            name: concat!("event at ", file!(), ":", line!()),
            target: $target,
            level: $lvl,
            fields: &[ "message", $(stringify!($field_name)),* ]
        )
    });
    (@
        name: $name:expr,
        target: $target:expr,
        level: $lvl:expr,
        fields: $field_names:expr
    ) => ({
        use std::sync::{Once, atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering}};
        use $crate::{callsite, Metadata, subscriber::Interest};
        struct MyCallsite;
        static META: Metadata<'static> = {
            use $crate::*;
            metadata! {
                name: $name,
                target: $target,
                level: $lvl,
                fields: $field_names,
                callsite: &MyCallsite,
            }
        };
        static INTEREST: AtomicUsize = ATOMIC_USIZE_INIT;
        static REGISTRATION: Once = Once::new();
        impl MyCallsite {
            #[inline]
            fn interest(&self) -> Interest {
                match INTEREST.load(Ordering::Relaxed) {
                    0 => Interest::NEVER,
                    2 => Interest::ALWAYS,
                    _ => Interest::SOMETIMES,
                }
            }
        }
        impl callsite::Callsite for MyCallsite {
            fn add_interest(&self, interest: Interest) {
                let current_interest = self.interest();
                let interest = match () {
                    // If the added interest is `NEVER`, don't change anything
                    // --- either a different subscriber added a higher
                    // interest, which we want to preserve, or the interest is 0
                    // anyway (as it's initialized to 0).
                    _ if interest.is_never() => return,
                    // If the interest is `SOMETIMES`, that overwrites a `NEVER`
                    // interest, but doesn't downgrade an `ALWAYS` interest.
                    _ if interest.is_sometimes() && current_interest.is_never() => 1,
                    // If the interest is `ALWAYS`, we overwrite the current
                    // interest, as ALWAYS is the highest interest level and
                    // should take precedent.
                    _ if interest.is_always() => 2,
                    _ => return,
                };
                INTEREST.store(interest, Ordering::Relaxed);
            }
            fn remove_interest(&self) {
                INTEREST.store(0, Ordering::Relaxed);
            }
            fn metadata(&self) -> &Metadata {
                &META
            }
        }
        REGISTRATION.call_once(|| {
            callsite::register(&MyCallsite);
        });
        &MyCallsite
    })
}

/// Constructs a new span.
///
/// # Examples
///
/// Creating a new span with no fields:
/// ```
/// # #[macro_use]
/// # extern crate tokio_trace;
/// # fn main() {
/// let mut span = span!("my span");
/// span.enter(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
///
/// Creating a span with fields:
/// ```
/// # #[macro_use]
/// # extern crate tokio_trace;
/// # fn main() {
/// span!("my span", foo = 2u64, bar = "a string").enter(|| {
///     // do work inside the span...
/// });
/// # }
/// ```
#[macro_export]
macro_rules! span {
    ($name:expr) => { span!($name,) };
    ($name:expr, $($k:ident $( = $val:expr )* ) ,*) => {
        {
            #[allow(unused_imports)]
            use $crate::{callsite, field::{Value, AsField}, Span};
            use $crate::callsite::Callsite;
            let callsite = callsite! { span: $name, $( $k ),* };
            let mut span = Span::new(callsite.interest(), callsite.metadata());
            // Depending on how many fields are generated, this may or may
            // not actually be used, but it doesn't make sense to repeat it.
            #[allow(unused_variables, unused_mut)] {
                if !span.is_disabled() {
                    let mut keys = callsite.metadata().fields().into_iter();
                    $(
                        let key = keys.next()
                            .expect(concat!("metadata should define a key for '", stringify!($k), "'"));
                        span!(@ record: span, $k, &key, $($val)*);
                    )*
                };
            }
            span
        }
    };
    (@ record: $span:expr, $k:expr, $i:expr, $val:expr) => (
        $span.record($i, &$val)
    );
    (@ record: $span:expr, $k:expr, $i:expr,) => (
        // skip
    );
}

#[macro_export]
macro_rules! event {
    // (target: $target:expr, $lvl:expr, { $( $k:ident $( = $val:expr )* ),* }, $fmt:expr ) => (
    //     event!(target: $target, $lvl, { $($k $( = $val)* ),* }, $fmt, )
    // );
    (target: $target:expr, $lvl:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => ({
        {
            #[allow(unused_imports)]
            use $crate::{callsite, Id, Subscriber, Event, field::{Value, AsField}};
            use $crate::callsite::Callsite;
            let callsite = callsite! { event:
                $lvl,
                target:
                $target, $( $k ),*
            };
            let mut event = Event::new(callsite.interest(), callsite.metadata());
            // Depending on how many fields are generated, this may or may
            // not actually be used, but it doesn't make sense to repeat it.
            #[allow(unused_variables, unused_mut)] {
                if !event.is_disabled() {
                    let mut keys = callsite.metadata().fields().into_iter();
                    let msg_key = keys.next()
                        .expect("event metadata should define a key for the message");
                    event.message(&msg_key, format_args!( $($arg)+ ));
                    $(
                        let key = keys.next()
                            .expect(concat!("metadata should define a key for '", stringify!($k), "'"));
                        event!(@ record: event, $k, &key, $($val)*);
                    )*
                }
            }
            event
        }
    });
    (target: $target:expr, $lvl:expr, { $( $k:ident $( = $val:expr )* ),* } ) => ({
        {
            #[allow(unused_imports)]
            use $crate::{callsite, Id, Subscriber, Event, field::{Value, AsField}};
            use $crate::callsite::Callsite;
            let callsite = callsite! { event:
                $lvl,
                target:
                $target, $( $k ),*
            };
            let mut event = Event::new(callsite.interest(), callsite.metadata());
            // Depending on how many fields are generated, this may or may
            // not actually be used, but it doesn't make sense to repeat it.
            #[allow(unused_variables, unused_mut)] {
                if !event.is_disabled() {
                    let mut keys = callsite.metadata().fields().into_iter();
                    let msg_key = keys.next()
                        .expect("event metadata should define a key for the message");
                    $(
                        let key = keys.next()
                            .expect(concat!("metadata should define a key for '", stringify!($k), "'"));
                        event!(@ record: event, $k, &key, $($val)*);
                    )*
                }
            }
            event
        }
    });
    ( $lvl:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: module_path!(), $lvl, { $($k $( = $val)* ),* }, $($arg)+)
    );
    ( $lvl:expr, $($arg:tt)+ ) => (
        event!(target: module_path!(), $lvl, { }, $($arg)+)
    );
    (@ record: $ev:expr, $k:expr, $i:expr, $val:expr) => (
        $ev.record($i, &$val);
    );
    (@ record: $ev:expr, $k:expr, $i:expr,) => (
        // skip
    );
}

#[macro_export]
macro_rules! trace {
    (target: $target:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: $target, $crate::Level::TRACE, { $($k $( = $val)* ),* }, $($arg)+)
    );
    (target: $target:expr, $( $k:ident $( = $val:expr )* ),* ) => (
        event!(target: $target, $crate::Level::TRACE, { $($k $( = $val)* ),* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        drop(event!(target: $target, $crate::Level::TRACE, {}, $($arg)+));
    );
    ({ $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        trace!(target: module_path!(), { $($k $( = $val)* ),* }, $($arg)+)
    );
    ($( $k:ident $( = $val:expr )* ),* ) => (
        trace!(target: module_path!(), { $($k $( = $val)* ),* })
    );
    ($($arg:tt)+ ) => (
        trace!(target: module_path!(), {}, $($arg)+)
    );
}

#[macro_export]
macro_rules! debug {
    (target: $target:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: $target, $crate::Level::DEBUG, { $($k $( = $val)* ),* }, $($arg)+)
    );
    (target: $target:expr, $( $k:ident $( = $val:expr )* ),* ) => (
        event!(target: $target, $crate::Level::DEBUG, { $($k $( = $val)* ),* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        drop(event!(target: $target, $crate::Level::DEBUG, {}, $($arg)+));
    );
    ({ $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        debug!(target: module_path!(), { $($k $( = $val)* ),* }, $($arg)+)
    );
    ($( $k:ident $( = $val:expr )* ),* ) => (
        debug!(target: module_path!(), { $($k $( = $val)* ),* })
    );
    ($($arg:tt)+ ) => (
        debug!(target: module_path!(), {}, $($arg)+)
    );
}

#[macro_export]
macro_rules! info {
    (target: $target:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: $target, $crate::Level::INFO, { $($k $( = $val)* ),* }, $($arg)+)
    );
    (target: $target:expr, $( $k:ident $( = $val:expr )* ),* ) => (
        event!(target: $target, $crate::Level::INFO, { $($k $( = $val)* ),* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        drop(event!(target: $target, $crate::Level::INFO, {}, $($arg)+));
    );
    ({ $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        info!(target: module_path!(), { $($k $( = $val)* ),* }, $($arg)+)
    );
    ($( $k:ident $( = $val:expr )* ),* ) => (
        info!(target: module_path!(), { $($k $( = $val)* ),* })
    );
    ($($arg:tt)+ ) => (
        info!(target: module_path!(), {}, $($arg)+)
    );
}

#[macro_export]
macro_rules! warn {
    (target: $target:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: $target, $crate::Level::WARN, { $($k $( = $val)* ),* }, $($arg)+)
    );
    (target: $target:expr, $( $k:ident $( = $val:expr )* ),* ) => (
        event!(target: $target, $crate::Level::WARN, { $($k $( = $val)* ),* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        drop(event!(target: $target, $crate::Level::WARN, {}, $($arg)+));
    );
    ({ $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        warn!(target: module_path!(), { $($k $( = $val)* ),* }, $($arg)+)
    );
    ($( $k:ident $( = $val:expr )* ),* ) => (
        warn!(target: module_path!(), { $($k $( = $val)* ),* })
    );
    ($($arg:tt)+ ) => (
        warn!(target: module_path!(), {}, $($arg)+)
    );
}

#[macro_export]
macro_rules! error {
    (target: $target:expr, { $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        event!(target: $target, $crate::Level::ERROR, { $($k $( = $val)* ),* }, $($arg)+)
    );
    (target: $target:expr, $( $k:ident $( = $val:expr )* ),* ) => (
        event!(target: $target, $crate::Level::ERROR, { $($k $( = $val)* ),* })
    );
    (target: $target:expr, $($arg:tt)+ ) => (
        drop(event!(target: $target, $crate::Level::ERROR, {}, $($arg)+));
    );
    ({ $( $k:ident $( = $val:expr )* ),* }, $($arg:tt)+ ) => (
        error!(target: module_path!(), { $($k $( = $val)* ),* }, $($arg)+)
    );
    ($( $k:ident $( = $val:expr )* ),* ) => (
        error!(target: module_path!(), { $($k $( = $val)* ),* })
    );
    ($($arg:tt)+ ) => (
        error!(target: module_path!(), {}, $($arg)+)
    );
}

pub mod dispatcher;
pub mod field;
pub mod span;
pub mod subscriber;

mod sealed {
    pub trait Sealed {}
}
