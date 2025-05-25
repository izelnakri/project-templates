//// TODO: struct NetworkMockTestCase::acquire() then figure out how to turn into a macro
use std::sync::{Mutex, MutexGuard};

/// Global mutex to synchronize tests that modify shared global state.
/// This ensures tests that mock network calls don't interfere with each other.
pub static GLOBAL_MOCK_NETWORK_TESTS_MUTEX: Mutex<()> = Mutex::new(());

/// A RAII guard that holds the global test mutex for the duration of a test.
/// When dropped, it automatically releases the mutex and cleans up any test state.
pub struct NetworkMockTestCase<'a> {
    _guard: MutexGuard<'a, ()>,
}

impl<'a> NetworkMockTestCase<'a> {
    pub fn register() -> NetworkMockTestCase<'a> {
        let guard = GLOBAL_MOCK_NETWORK_TESTS_MUTEX.lock().unwrap();
        NetworkMockTestCase { _guard: guard }
    }
}

impl<'a> Drop for NetworkMockTestCase<'a> {
    fn drop(&mut self) {
        // Clean up any global state when the guard is dropped
        github_user_fetcher::user::set_base_url_override(None); // NOTE: Can I make this less specific?
    }
}
