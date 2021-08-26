//! A simple utility to take panic payloads (of type `Box<dyn Any + Send + 'static>`), primarily
//! obtained from [`std::panic::catch_unwind`], and converting them into messages (`&str`'s)
//!
//! There are two primary entrypoints:
//!
//! `panic_message` returns a `&str` and will do its best attempt to unpack a `&str` message from
//! the payload, but will default to the literal `"Box<dyn Any>"` in an attempt to recreate what
//! rustc does by default:
//!
//! ```
//! use std::panic::catch_unwind;
//!
//! let payload = catch_unwind(|| {
//!     panic!("gus");
//! }).unwrap_err();
//!
//! let msg = panic_message::panic_message(&payload);
//! assert_eq!("gus", msg);
//! ```
//! Non-string payload:
//! ```
//! use std::panic::catch_unwind;
//!
//! let payload = catch_unwind(|| {
//!     std::panic::panic_any(1);
//! }).unwrap_err();
//!
//! let msg = panic_message::panic_message(&payload);
//! assert_eq!("Box<dyn Any>", msg);
//! ```
//!
//! `get_panic_message` is similar, but returns an `Option<&str>`, returning `None` when it can't
//! unpack a message from the payload
//!
//! ```
//! use std::panic::catch_unwind;
//!
//! let payload = catch_unwind(|| {
//!     panic!("gus");
//! }).unwrap_err();
//!
//! let msg = panic_message::get_panic_message(&payload);
//! assert_eq!(Some("gus"), msg);
//! ```
//! Non-string payload:
//! ```
//! use std::panic::catch_unwind;
//!
//! let payload = catch_unwind(|| {
//!     std::panic::panic_any(1);
//! }).unwrap_err();
//!
//! let msg = panic_message::get_panic_message(&payload);
//! assert_eq!(None, msg);
//! ```
//!
//!
//! # Note
//! This library takes in `&Box<dyn Any + Send + 'static>`. This is to make it clear that its
//! borrowing from a full panic payload. This is to avoid misuse from:
//! - Passing some other `&dyn Any` value
//! - Coercion issues from `Box<dyn Any` to a `&dyn Any` that represents the `Box` itself
//!
use std::any::Any;

/// Produce a `&str` message from a panic payload, with a default message.
/// See [module docs][crate] for usage.
pub fn panic_message(payload: &Box<dyn Any + Send>) -> &str {
    get_panic_message(payload).unwrap_or({
        // Copy what rustc does in the default panic handler
        "Box<dyn Any>"
    })
}

/// Attempt to produce a `&str` message from a panic payload.
/// See [module docs][crate] for usage.
pub fn get_panic_message(payload: &Box<dyn Any + Send>) -> Option<&str> {
    // taken from: https://github.com/rust-lang/rust/blob/4b9f4b221b92193c7e95b1beb502c6eb32c3b613/library/std/src/panicking.rs#L194-L200
    match payload.as_ref().downcast_ref::<&'static str>() {
        Some(msg) => Some(*msg),
        None => match payload.as_ref().downcast_ref::<String>() {
            Some(msg) => Some(msg.as_str()),
            // Copy what rustc does in the default panic handler
            None => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn basic() {
        let payload = catch_unwind(|| panic!("gus")).unwrap_err();

        let msg = panic_message(&payload);

        assert_eq!("gus", msg);
    }

    #[test]
    fn string() {
        let payload = catch_unwind(|| std::panic::panic_any("gus".to_string())).unwrap_err();

        let msg = panic_message(&payload);

        assert_eq!("gus", msg);
    }

    #[test]
    fn expect() {
        let payload = catch_unwind(|| {
            // Note this is a reference to a local string
            // but expect internally turns it back into a String for the payload
            None::<()>.expect(&format!("{}", "gus"))
        })
        .unwrap_err();

        let msg = panic_message(&payload);

        assert_eq!("gus", msg);
    }

    #[test]
    fn something_else() {
        let payload = catch_unwind(|| {
            std::panic::panic_any(1);
        })
        .unwrap_err();

        let msg = panic_message(&payload);

        assert_eq!("Box<dyn Any>", msg);
    }
}
