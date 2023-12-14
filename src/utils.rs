/// if a thread panics we kill the whole process
pub fn set_panic_hook() {
    let orig_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        std::process::exit(1);
    }));
}
