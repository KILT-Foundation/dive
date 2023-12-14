use actix_rt::System;

/// if a thread panics we kill the whole process gracefully
pub fn set_panic_hook() {
    let orig_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // shut down the other threads gracefully
        System::current().stop();

        // continue to propagate the panic.
        orig_hook(panic_info);
    }));
}
