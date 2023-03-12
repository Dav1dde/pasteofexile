use itertools::Itertools;

#[derive(Debug, Default)]
pub struct PercentRoute<T>(pub T);

impl<T> std::ops::Deref for PercentRoute<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: sycamore_router::Route + std::fmt::Debug + Default> sycamore_router::Route
    for PercentRoute<T>
{
    fn match_route(&self, segments: &[&str]) -> Self {
        let segments = segments
            .iter()
            .map(|segment| percent_encoding::percent_decode_str(segment).decode_utf8_lossy())
            .collect_vec();
        let segments_ref = segments.iter().map(|s| s.as_ref()).collect_vec();
        Self(T::match_route(&T::default(), &segments_ref))
    }

    fn match_path(&self, path: &str) -> Self {
        // We actually have to decode each segment separately to make sure we don't decode a slash
        // too early.
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        self.match_route(&segments)
    }
}
