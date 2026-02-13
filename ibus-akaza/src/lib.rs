#![allow(non_upper_case_globals)]

// Expose modules for testing
pub mod commands;
pub mod context;
pub mod current_state;
pub mod input_mode;
pub mod keymap;
pub mod ui;
pub mod wrapper_bindings;

pub mod test_utils {
    use ibus_sys::engine::IBusEngine;
    use std::ptr;

    /// Mock IBusEngine for integration tests
    /// Returns a null pointer that should not be dereferenced
    pub fn mock_engine() -> *mut IBusEngine {
        ptr::null_mut()
    }
}
