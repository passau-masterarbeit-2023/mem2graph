use std::sync::Once;

// reference tests from tests/ directory
#[cfg(test)]
mod utils;
mod graph_structs;

static INIT: Once = Once::new();

/// WARN: Must be called after init()
/// otherwise, the logger will not be initialized
fn log_order_warning() {
    INIT.call_once(|| {
        log::info!(" 🚧 The order of the logs is not guaranteed. This is because the tests are run in parallel.");
        log::info!(" 🚧 Using 'print' or 'println' won't work because the output is captured by the test runner.");
    });
}


// setup() function is called before each test
pub fn setup() {
        // initialization code here
        crate::params::init(); // Call the init() function to load the .env file
        log_order_warning();
}
