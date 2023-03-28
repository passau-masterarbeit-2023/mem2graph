// reference tests from tests/ directory
#[cfg(test)]
mod utils;


// setup() function is called before each test
pub fn setup() {
        // initialization code here
        crate::params::init(); // Call the init() function to load the .env file
}
