use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// 连接限制器
///
/// 使用信号量限制并发连接数
#[derive(Clone)]
pub struct ConnectionLimiter {
    semaphore: Arc<Semaphore>,
    active_connections: Arc<AtomicUsize>,
    max_connections: usize,
}

impl ConnectionLimiter {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            active_connections: Arc::new(AtomicUsize::new(0)),
            max_connections,
        }
    }

    /// 获取连接许可
    pub async fn acquire(&self) -> Option<ConnectionGuard> {
        match self.semaphore.clone().acquire_owned().await {
            Ok(permit) => {
                self.active_connections.fetch_add(1, Ordering::Relaxed);
                Some(ConnectionGuard {
                    _permit: permit,
                    active_connections: self.active_connections.clone(),
                })
            }
            Err(_) => None,
        }
    }

    /// 获取当前活跃连接数
    pub fn active_count(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// 获取最大连接数
    pub fn max_count(&self) -> usize {
        self.max_connections
    }
}

/// 连接守卫
///
/// 当守卫被丢弃时，自动释放连接许可
pub struct ConnectionGuard {
    _permit: OwnedSemaphorePermit,
    active_connections: Arc<AtomicUsize>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_limiter_creation() {
        let limiter = ConnectionLimiter::new(10);
        assert_eq!(limiter.max_count(), 10);
        assert_eq!(limiter.active_count(), 0);
    }

    #[tokio::test]
    async fn test_connection_limiter_acquire() {
        let limiter = ConnectionLimiter::new(2);

        let guard1 = limiter.acquire().await;
        assert!(guard1.is_some());
        assert_eq!(limiter.active_count(), 1);

        let guard2 = limiter.acquire().await;
        assert!(guard2.is_some());
        assert_eq!(limiter.active_count(), 2);
    }

    #[tokio::test]
    async fn test_connection_limiter_release() {
        let limiter = ConnectionLimiter::new(2);

        {
            let _guard1 = limiter.acquire().await;
            assert_eq!(limiter.active_count(), 1);
        }

        // Guard dropped, count should decrease
        assert_eq!(limiter.active_count(), 0);
    }

    #[tokio::test]
    async fn test_connection_limiter_max_connections() {
        let limiter = ConnectionLimiter::new(2);

        let _guard1 = limiter.acquire().await.unwrap();
        let _guard2 = limiter.acquire().await.unwrap();

        assert_eq!(limiter.active_count(), 2);

        // Third connection should wait (we'll test with timeout)
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            limiter.acquire()
        ).await;

        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_connection_limiter_clone() {
        let limiter = ConnectionLimiter::new(5);
        let limiter_clone = limiter.clone();

        let _guard = limiter.acquire().await.unwrap();
        assert_eq!(limiter_clone.active_count(), 1);
    }
}
