use std::future::Future;
use std::pin::Pin;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
