pub mod mutex_wrapper {
    #[cfg(features = "parking_lot")]
    use parking_lot::Mutex;

    #[cfg(not(features = "parking_lot"))]
    use std::sync::Mutex;

    pub struct MutexWrapper<T: ?Sized>(Mutex<T>);

    impl<T> MutexWrapper<T> {
        pub fn new(data: T) -> Self {
            Self(Mutex::new(data))
        }

        #[cfg(features = "parking_lot")]
        pub fn lock(&self) -> lock_api::MutexGuard<T> {
            self.0.lock()
        }

        #[cfg(not(features = "parking_lot"))]
        pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
            self.0.lock().expect("acquire the lock")
        }
    }
}
