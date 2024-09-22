use prometheus::{Encoder, HistogramOpts, HistogramVec, TextEncoder};
use prometheus::{IntCounter, IntGauge};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use warp::Filter;

#[derive(Clone)]
pub struct Container {
  cache_gauge: IntGauge,
  cache_add: IntCounter,
  task_times: HistogramVec,
}

impl Container {
  pub fn cache(&self, num_items: i64) {
    self.cache_gauge.set(num_items);
  }
  pub fn cache_add(&self) {
    self.cache_add.inc();
  }
  pub fn measure(&self, start: Instant, func: &str) {
    let duration = start.elapsed();
    self.task_times.with_label_values(&[func]).observe(duration.as_secs_f64());
  }
}

impl Container {
  pub fn new() -> anyhow::Result<Self> {
    let m = Self {
      cache_gauge: IntGauge::new("name_cache", "Cache Entries")?,
      cache_add: IntCounter::new("name_add", "Cache item added")?,
      task_times: HistogramVec::new(HistogramOpts::new("task_time", "Task Timers"), &["function"])?,
    };
    prometheus::default_registry().register(Box::new(m.cache_gauge.clone()))?;
    prometheus::default_registry().register(Box::new(m.cache_add.clone()))?;
    prometheus::default_registry().register(Box::new(m.task_times.clone()))?;
    Ok(m)
  }
}

#[allow(dead_code)]
pub(crate) fn with_metrics(
  metrics_container: Arc<Container>,
) -> impl Filter<Extract = (Arc<Container>,), Error = Infallible> + Clone {
  warp::any().map(move || metrics_container.clone())
}

pub(crate) async fn handler() -> Result<impl warp::Reply, warp::Rejection> {
  let encoder = TextEncoder::new();
  let metric_families = prometheus::gather();
  let mut buffer = Vec::new();
  encoder.encode(&metric_families, &mut buffer).unwrap();
  Ok(warp::reply::with_header(buffer, "Content-Type", encoder.format_type()))
}
