use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use std::cmp::min;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tower::{Layer, Service};
// use tower::Layer;

// This is just a temp fix
#[derive(Debug)]
pub struct RateLimitError {
    pub msg: String,
}

// #[derive(Clone)]
// Impl Clone manually
pub struct RateLimiterService<S, A> {
    inner: S,
    algorithm: Arc<A>,
}

impl<S: Clone, A> Clone for RateLimiterService<S, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            algorithm: Arc::clone(&self.algorithm),
        }
    }
}

// i have to pass this in axum layer right !
// it must be cheaply cloneable
// thats wehy arc
// #[derive(Clone)]
// consider manual impl clone for
pub struct RateLimiterLayer<A> {
    pub algorithm: Arc<A>,
}
impl<A> RateLimiterLayer<A> {
    pub fn new(algorithm: A) -> Self {
        RateLimiterLayer {
            algorithm: Arc::new(algorithm),
        }
    }
}

impl<A> Clone for RateLimiterLayer<A> {
    fn clone(&self) -> Self {
        Self {
            algorithm: Arc::clone(&self.algorithm),
        }
    }
}

impl<S, A> Layer<S> for RateLimiterLayer<A> {
    type Service = RateLimiterService<S, A>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimiterService {
            inner,
            algorithm: Arc::clone(&self.algorithm),
        }
    }
}

pub trait RateLimitAlgorithm: Send + Sync + 'static {
    fn try_acquire(&self) -> impl Future<Output = bool> + Send;
}

impl RateLimitAlgorithm for TokenBucket {
    async fn try_acquire(&self) -> bool {
        self.try_acquire_one(1).await
    }
}

impl<S, A> Service<Request<Body>> for RateLimiterService<S, A>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    A: RateLimitAlgorithm,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let algorithm = Arc::clone(&self.algorithm);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if algorithm.try_acquire().await {
                inner.call(req).await
            } else {
                Ok((StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response())
            }
        })
    }
}

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

    pub async fn acquire_one(&self) -> bool {
        // worst case infinite loop
        loop {
            let mut guard = self.inner.lock().await;

            guard.refill(self.refill_rate, self.bucket_size);

            if guard.tokens > 0 {
                guard.tokens -= 1;
                return true;
            } else {
                let wait_time = Duration::from_secs_f64(1.0 / self.refill_rate as f64);

                drop(guard);

                sleep(wait_time).await;
            }
        }
    }

    pub async fn try_acquire_one(&self, _n: u64) -> bool {
        // acquire or false
        let mut guard = self.inner.lock().await;

        guard.refill(self.refill_rate, self.bucket_size);

        if guard.tokens > 0 {
            guard.tokens -= 1;
            true
        } else {
            false
        }
    }

    pub async fn available_tokens(&self) -> u64 {
        let mut guard = self.inner.lock().await;

        guard.refill(self.refill_rate, self.bucket_size);

        let tokens = guard.tokens;

        drop(guard);

        tokens
    }
}

impl TokenBucketState {
    pub fn refill(&mut self, refill_rate: u64, bucket_size: u64) {
        let now = Instant::now();
        let elapsed_duration = now - self.last_update;

        let newly_generated_tokens = elapsed_duration.as_secs() * refill_rate;

        self.tokens = min(self.tokens + newly_generated_tokens, bucket_size);

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
