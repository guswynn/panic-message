//! This test is an integration test because it messes with `assert`'s in
//! a `set_hook` handler, which can race with other tests, so it must be run on its own
//!
use std::panic::{catch_unwind, set_hook};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use panic_message::*;

#[test]
fn panic_info() {
    static CALLED: AtomicBool = AtomicBool::new(false);

    set_hook(Box::new(|pi| {
        // assert's here will SIGILL or abort the process if they fail
        assert_eq!("gus", panic_info_message(pi));
        assert_eq!(Some("gus"), get_panic_info_message(pi));
        CALLED.store(true, SeqCst);
    }));

    let payload = catch_unwind(|| {
        panic!("gus");
    })
    .unwrap_err();

    let msg = panic_message(&payload);
    assert_eq!("gus", msg);
    // Ensure we actually entered the hook
    assert!(CALLED.load(SeqCst));
}
