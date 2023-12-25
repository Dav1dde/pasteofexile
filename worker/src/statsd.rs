pub enum Counters {
    Request,
    RequestError,
    CacheHit,
    StorageGet,
    StorageDelete,
    StoragePut,
    StorageList,
    Fetch,
    PobUpload,
}

impl sentry::MetricName for Counters {
    fn name(&self) -> &'static str {
        match self {
            Counters::Request => "request.total",
            Counters::RequestError => "request.error",
            Counters::CacheHit => "request.cache.hit",
            Counters::StorageGet => "storage.get",
            Counters::StorageDelete => "storage.delete",
            Counters::StoragePut => "storage.put",
            Counters::StorageList => "storage.list",
            Counters::Fetch => "fetch.total",
            Counters::PobUpload => "pob.upload",
        }
    }
}

pub enum Distributions {
    PobSize,
}

impl sentry::MetricName for Distributions {
    fn name(&self) -> &'static str {
        match self {
            Distributions::PobSize => "pob.size",
        }
    }
}
