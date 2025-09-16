use ruffle_core::backend::log::LogBackend;

#[derive(Clone)]
pub struct ConsoleLogBackend {}

impl ConsoleLogBackend {
    pub fn new() -> ConsoleLogBackend
    {
        return ConsoleLogBackend{};
    }
}

impl LogBackend for ConsoleLogBackend {
    fn avm_trace(&self, message: &str) {
        println!("{}", message);
    }

    fn avm_warning(&self, message: &str) {
        // Match the format used by Flash Player
        println!("Warning: {}", message);
    }
}
