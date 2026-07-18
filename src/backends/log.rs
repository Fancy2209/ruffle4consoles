use ruffle_core::backend::log::LogBackend;

#[derive(Clone)]
pub struct ConsoleLogBackend {}

impl Default for ConsoleLogBackend {
    fn default() -> Self {
        Self {}
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
