use napi::bindgen_prelude::BigInt;
use opendal::raw::parse_datetime_from_from_timestamp_millis;
use std::ops::Bound as RangeBound;

#[napi(object)]
#[derive(Default)]
pub struct ReadOptions {
    pub version: Option<String>,
    pub concurrent: Option<u32>,
    pub chunk: Option<u32>,
    pub gap: Option<BigInt>,
    pub offset: Option<BigInt>,
    pub size: Option<BigInt>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<i64>,
    pub if_unmodified_since: Option<i64>,
    pub content_type: Option<String>,
    pub cache_control: Option<String>,
    pub content_disposition: Option<String>,
}

impl ReadOptions {
    pub fn make_range(&self) -> (RangeBound<u64>, RangeBound<u64>) {
        let start_bound = self.offset.clone().map_or(RangeBound::Unbounded, |s| {
            RangeBound::Included(s.get_u64().1)
        });
        let end_bound = self.size.clone().map_or(RangeBound::Unbounded, |e| {
            RangeBound::Excluded(e.get_u64().1)
        });

        (start_bound, end_bound)
    }
}

impl From<ReadOptions> for opendal::options::ReadOptions {
    fn from(opts: ReadOptions) -> Self {
        let r = opts.make_range();
        Self {
            version: opts.version,
            concurrent: opts.concurrent.unwrap_or_default() as usize,
            chunk: Some(opts.chunk.unwrap_or_default() as usize),
            gap: opts.gap.map(|gap| gap.get_u64().1 as usize),
            range: r.into(),
            if_match: opts.if_match,
            if_none_match: opts.if_none_match,
            if_modified_since: opts
                .if_modified_since
                .and_then(|timestamp| parse_datetime_from_from_timestamp_millis(timestamp).ok()),
            if_unmodified_since: opts
                .if_unmodified_since
                .and_then(|timestamp| parse_datetime_from_from_timestamp_millis(timestamp).ok()),
            override_content_type: opts.content_type,
            override_cache_control: opts.cache_control,
            override_content_disposition: opts.content_disposition,
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct ReaderOptions {
    pub version: Option<String>,
    pub concurrent: Option<u32>,
    pub chunk: Option<u32>,
    pub gap: Option<BigInt>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<i64>,
    pub if_unmodified_since: Option<i64>,
}

impl From<ReaderOptions> for opendal::options::ReaderOptions {
    fn from(opts: ReaderOptions) -> Self {
        Self {
            version: opts.version,
            concurrent: opts.concurrent.unwrap_or_default() as usize,
            chunk: Some(opts.chunk.unwrap_or_default() as usize),
            gap: opts.gap.map(|gap| gap.get_u64().1 as usize),
            if_match: opts.if_match,
            if_none_match: opts.if_none_match,
            if_modified_since: opts
                .if_modified_since
                .and_then(|timestamp| parse_datetime_from_from_timestamp_millis(timestamp).ok()),
            if_unmodified_since: opts
                .if_unmodified_since
                .and_then(|timestamp| parse_datetime_from_from_timestamp_millis(timestamp).ok()),
        }
    }
}
