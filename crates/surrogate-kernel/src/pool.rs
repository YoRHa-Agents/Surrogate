use crate::KernelError;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ConnectionPool {
    max_active: usize,
    active: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct PoolGuard {
    active: Arc<AtomicUsize>,
}

impl Drop for PoolGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

impl ConnectionPool {
    pub fn new(max_active: usize) -> Self {
        Self {
            max_active,
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn try_acquire(&self) -> Result<PoolGuard, KernelError> {
        loop {
            let current = self.active.load(Ordering::SeqCst);
            if current >= self.max_active {
                return Err(KernelError::PoolExhausted);
            }
            if self
                .active
                .compare_exchange_weak(current, current + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(PoolGuard {
                    active: Arc::clone(&self.active),
                });
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_acquire_release() {
        let pool = ConnectionPool::new(2);
        assert_eq!(pool.active_count(), 0);

        let guard1 = pool.try_acquire().expect("first acquire");
        assert_eq!(pool.active_count(), 1);

        let guard2 = pool.try_acquire().expect("second acquire");
        assert_eq!(pool.active_count(), 2);

        drop(guard1);
        assert_eq!(pool.active_count(), 1);

        drop(guard2);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn pool_exhaustion() {
        let pool = ConnectionPool::new(1);

        let _guard = pool.try_acquire().expect("first acquire should succeed");
        assert_eq!(pool.active_count(), 1);

        let result = pool.try_acquire();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KernelError::PoolExhausted));
    }
}
