//! A simple utility to take panic payloads, primarily obtained from
//! obtained from [`std::panic::catch_unwind`] or [`std::panic::set_hook`],
//! and converting them into messages (`&str`'s)
//!
//! # `panic_message`
//!
//! [`panic_message`][crate::panic_message] takes a payload from `[std::panic::catch_unwind`] and returns a `&str`,
//! doing its best attempt to unpack a `&str` message from the payload, defaulting to the
//! literal `"Box<dyn Any>"` in an attempt to recreate what rustc does.
//!
//! ## Examples
//! ```
//! use std::panic::catch_unwind;
//!
//! let payload = catch_unwind(|| {
//!     panic!("gus"); }).unwrap_err();
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
//! # `get_panic_message`
//!
//! [`get_panic_message`][crate::get_panic_message] is similar to `panic_message`, but returns an `Option<&str>`,
//! returning `None` when it can't unpack a message from the payload
//!
//! ## Examples
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
//! # `PanicInfo`
//!
//! This library also offers apis for getting messages from [`PanicInfo`][std::panic::PanicInfo`]'s
//! as returned by [`std::panic::set_hook`]:
//! - [`panic_info_message`][crate::panic_info_message] is similar
//! to [`panic_message`][crate::panic_message] and has a default string `"Box<dyn Any>"`
//! - [`get_panic_info_message`][crate::get_panic_info_message] is similar
//! to [`get_panic_message`][crate::get_panic_message] and returns an `Option<&str>`
//!
//! ## Example
//!
//! ```
//! std::panic::set_hook(Box::new(|pi| {
//!     println!("{}", panic_message::panic_info_message(pi));
//!     println!("{:?}", panic_message::get_panic_info_message(pi));
//! }));
//! ```
//!
//! # Note
//!
//! This library has methods that take values that are returned by standard mechanisms to obtain
//! panic payloads, as opposed to a single generic method that takes `&dyn Any`.
//! This is to prevent misuse.
//! For example, the reason to take `PanicInfo` and not the `&dyn Any` as returned by
//! [`PanicInfo::payload`][std::panic::PanicInfo::payload] is because `Box<dyn Any>`
//! can be coerced into `&dyn Any`, which would make a method that takes `&dyn Any` possible
//! to misuse with a payload from [`std::panic::catch_unwind`].
//!
use std::{any::Any, panic::PanicInfo};

/// Attempt to produce a `&str` message (with a default)
/// from a [`std::panic::catch_unwind`] payload.
/// See [module docs][crate] for usage.
pub fn panic_message(payload: &Box<dyn Any + Send>) -> &str {
    imp::get_panic_message(payload.as_ref()).unwrap_or({
        // Copy what rustc does in the default panic handler
        "Box<dyn Any>"
    })
}

/// Attempt to produce a `&str` message
/// from a [`std::panic::catch_unwind`] payload.
/// See [module docs][crate] for usage.
pub fn get_panic_message(payload: &Box<dyn Any + Send>) -> Option<&str> {
    imp::get_panic_message(payload.as_ref())
}

/// Attempt to produce a `&str` message (with a default)
/// from a [`std::panic::PanicInfo`].
/// See [module docs][crate] for usage.
pub fn panic_info_message<'pi>(panic_info: &'pi PanicInfo<'_>) -> &'pi str {
    imp::get_panic_message(panic_info.payload()).unwrap_or({
        // Copy what rustc does in the default panic handler
        "Box<dyn Any>"
    })
}

/// Attempt to produce a `&str` message (with a default)
/// from a [`std::panic::PanicInfo`].
/// See [module docs][crate] for usage.
pub fn get_panic_info_message<'pi>(panic_info: &'pi PanicInfo<'_>) -> Option<&'pi str> {
    imp::get_panic_message(panic_info.payload())
}

mod imp {
    use super::*;
    /// Attempt to produce a message from a borrowed `dyn Any`. Note that care must be taken
    /// when calling this to avoid a `Box<dyn Any>` being coerced to a `dyn Any` itself.
    pub(super) fn get_panic_message(payload: &dyn Any) -> Option<&str> {
        // taken from: https://github.com/rust-lang/rust/blob/4b9f4b221b92193c7e95b1beb502c6eb32c3b613/library/std/src/panicking.rs#L194-L200
        match payload.downcast_ref::<&'static str>() {
            Some(msg) => Some(*msg),
            None => match payload.downcast_ref::<String>() {
                Some(msg) => Some(msg.as_str()),
                // Copy what rustc does in the default panic handler
                None => None,
            },
        }
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
