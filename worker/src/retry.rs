use std::future::Future;

use futures::FutureExt;
use tracing::Span;

pub enum Retry<T, Err> {
    Ok(T),
    Err(Err),
}

impl<T, Err> Retry<T, Err> {
    pub fn ok(ok: T) -> Result<Self, Err> {
        Ok(Self::Ok(ok))
    }

    pub fn err(err: Err) -> Result<Self, Err> {
        Ok(Self::Err(err))
    }

    pub fn auto(result: Result<T, Err>) -> Result<Self, Err> {
        match result {
            Ok(value) => Self::ok(value),
            Err(err) => Self::err(err),
        }
    }
}

pub fn retry_all<F, T, Err, Fut>(
    max_attempts: usize,
    mut f: F,
) -> impl Future<Output = Result<T, Err>>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<T, Err>>,
    Err: std::fmt::Debug,
{
    retry(max_attempts, move |i| f(i).map(Retry::auto))
}

#[tracing::instrument(skip(f), fields(attempt))]
pub async fn retry<F, T, Err, Fut>(max_attempts: usize, mut f: F) -> Result<T, Err>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<Retry<T, Err>, Err>>,
    Err: std::fmt::Debug,
{
    assert!(
        max_attempts > 1,
        "it does not make sense to retry with only 1 attempt"
    );
    for i in 1..=max_attempts {
        Span::current().record("attempt", i);
        match f(i).await {
            Ok(Retry::Ok(result)) => return Ok(result),
            Ok(Retry::Err(err)) => {
                let is_last_attempt = i == max_attempts;
                if is_last_attempt {
                    tracing::warn!(err = ?err, "no more attempts available {max_attempts}/{max_attempts}");
                    return Err(err);
                }
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!();
}
