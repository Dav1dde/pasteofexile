use std::future::Future;

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
}

pub async fn retry<F, T, Err, Fut>(max_attempts: usize, mut f: F) -> Result<T, Err>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<Retry<T, Err>, Err>>,
{
    assert!(
        max_attempts > 1,
        "it does not make sense to retry with only 1 attempt"
    );
    for i in 1..=max_attempts {
        match f(i).await {
            Ok(Retry::Ok(result)) => return Ok(result),
            Ok(Retry::Err(err)) => {
                let is_last_attempt = i == max_attempts;
                if is_last_attempt {
                    log::warn!("no more attempts available");
                    return Err(err);
                }
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!();
}
