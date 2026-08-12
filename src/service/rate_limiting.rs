use std::cmp::min;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tower::Layer;

#[derive(Debug)]
pub struct RateLimitError {
    pub msg: String,
}

pub struct RateLimiterLayer<A> {
    pub algorithm: Arc<A>,
}

pub struct RateLimiterService<A, S> {
    pub inner: S,
    pub algorithm: Arc<A>,
}

impl<A> RateLimiterLayer<A> {
    pub fn new(algorithm: A) -> Self {
        Self {
            algorithm: Arc::new(algorithm),
        }
    }
}

impl<S, A> Layer<S> for RateLimiterLayer<A> {
    type Service = RateLimiterService<A, S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimiterService {
            inner,
            algorithm: Arc::clone(&self.algorithm),
        }
    }
}

// impl<S, A> Service<Request<Body>> for RateLimiterService<A, S>
// where
// S : Service<Request<Body>> , A :
// use inside Arc
pub struct TokenBucket {
    inner: Mutex<TokenBucketState>,
    bucket_size: u64,
    refill_rate: u64,
}

pub struct TokenBucketState {
    last_update: Instant,
    tokens: u64,
}

impl TokenBucket {
    pub fn new_empty(size: u64, rate: u64) -> Result<Self, RateLimitError> {
        if size == 0 || rate == 0 {
            return Err(RateLimitError {
                msg: "Size and Rate must be > 0".to_string(),
            });
        }

        let inner = Mutex::new(TokenBucketState {
            last_update: Instant::now(),
            tokens: 0,
        });

        Ok(TokenBucket {
            inner,
            bucket_size: size,
            refill_rate: rate,
        })
    }

    pub fn new(size: u64, rate: u64) -> Result<Self, RateLimitError> {
        if size == 0 || rate == 0 {
            return Err(RateLimitError {
                msg: "Size and Rate must be > 0".to_string(),
            });
        }

        let inner = Mutex::new(TokenBucketState {
            last_update: Instant::now(),
            tokens: size,
        });

        Ok(TokenBucket {
            inner,
            bucket_size: size,
            refill_rate: rate,
        })
    }

    pub async fn aquire_one(&self) -> bool {
        // worst case infinite loop
        loop {
            let mut gaurd = self.inner.lock().await;

            gaurd.refill(self.refill_rate, self.bucket_size);

            if gaurd.tokens > 0 {
                gaurd.tokens -= 1;
                return true;
            } else {
                let wait_time = Duration::from_secs_f64(1.0 / self.refill_rate as f64);

                drop(gaurd);

                sleep(wait_time).await;
            }
        }
    }

    pub async fn try_aquire_one(&self, _n: u64) -> bool {
        // aquire or false
        let mut gaurd = self.inner.lock().await;

        gaurd.refill(self.refill_rate, self.bucket_size);

        if gaurd.tokens > 0 {
            gaurd.tokens -= 1;
            true
        } else {
            false
        }
    }

    pub async fn available_tokens(&self) -> u64 {
        let mut gaurd = self.inner.lock().await;

        gaurd.refill(self.refill_rate, self.bucket_size);

        let tokens = gaurd.tokens;

        drop(gaurd);

        tokens
    }
}

impl TokenBucketState {
    pub fn refill(&mut self, refill_rate: u64, bucket_size: u64) {
        let now = Instant::now();
        let elapsed_duration = now - self.last_update;

        let newly_genrated_tokens = elapsed_duration.as_secs() * refill_rate;

        self.tokens = min(self.tokens + newly_genrated_tokens, bucket_size);

        self.last_update += Duration::from_secs(elapsed_duration.as_secs());
    }
}

pub struct LeakyBucket {}
pub struct FixedWindowCounter {}
pub struct SlidingWindowLog {}
pub struct SlidingWindowCounter {}

mod test {
    

    #[tokio::test]
    async fn new_token_bucket_zero_size_rate() {
        std::panic::set_hook(Box::new(|_| {}));

        let t_bucket = TokenBucket::new(0, 0);
        let t_empty_bucket = TokenBucket::new_empty(0, 0);

        assert!(t_bucket.is_err());
        assert!(t_empty_bucket.is_err());
    }

    #[tokio::test]
    async fn new_token_bucket() {
        let t_bucket = TokenBucket::new(10, 5);
        let t_empty_bucket = TokenBucket::new_empty(10, 5);

        assert!(t_bucket.is_ok());
        assert!(t_empty_bucket.is_ok());
    }
}
