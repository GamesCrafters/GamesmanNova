//! Histogram data structure and rendering.

use ordered_float::OrderedFloat;

use std::collections::BTreeMap;

/* CONSTANTS */

const HISTOGRAM_BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const MAX_CENTROIDS: usize = 10_000;

/* STRUCTURES */

/// Adaptive histogram using BTreeMap for O(log n) operations.
pub struct TickSketch {
    centroids: BTreeMap<OrderedFloat<f64>, u64>,
    capacity: usize,
}

/* IMPLEMENTATIONS */

impl Default for TickSketch {
    fn default() -> Self {
        Self::new(MAX_CENTROIDS)
    }
}

impl TickSketch {
    pub fn new(capacity: usize) -> Self {
        Self {
            centroids: BTreeMap::new(),
            capacity,
        }
    }

    /// Record a tick duration in nanoseconds.
    pub fn record(&mut self, tick_ns: u64) {
        let key = OrderedFloat(tick_ns as f64);

        *self
            .centroids
            .entry(key)
            .or_insert(0) += 1;

        if self.centroids.len() > self.capacity {
            self.merge_closest();
        }
    }

    pub fn resize(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
        while self.centroids.len() > self.capacity {
            self.merge_closest();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.centroids.is_empty()
    }

    pub fn min_ns(&self) -> u64 {
        self.centroids
            .first_key_value()
            .map(|(k, _)| k.0 as u64)
            .unwrap_or(1)
    }

    pub fn max_ns(&self) -> u64 {
        self.centroids
            .last_key_value()
            .map(|(k, _)| k.0 as u64)
            .unwrap_or(1)
    }

    pub fn centroid_values(&self) -> Vec<u64> {
        self.centroids
            .keys()
            .map(|k| k.0 as u64)
            .collect()
    }

    pub fn centroid_counts(&self) -> Vec<u64> {
        self.centroids
            .values()
            .copied()
            .collect()
    }

    fn merge_closest(&mut self) {
        if self.centroids.len() < 2 {
            return;
        }

        let (key1, key2) = self.find_closest_pair();
        let (merged_key, merged_count) = self.compute_merge(key1, key2);

        self.centroids.remove(&key1);
        self.centroids.remove(&key2);
        self.centroids.insert(merged_key, merged_count);
    }

    fn find_closest_pair(&self) -> (OrderedFloat<f64>, OrderedFloat<f64>) {
        self.centroids
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| ((w[1].0 / w[0].0).ln(), w[0], w[1]))
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, k1, k2)| (k1, k2))
            .unwrap()
    }

    fn compute_merge(
        &self,
        key1: OrderedFloat<f64>,
        key2: OrderedFloat<f64>,
    ) -> (OrderedFloat<f64>, u64) {
        let count1 = self.centroids.get(&key1).copied().unwrap_or(0);
        let count2 = self.centroids.get(&key2).copied().unwrap_or(0);

        let total = count1 + count2;
        let weight = (key1.0 * count1 as f64 + key2.0 * count2 as f64) / total as f64;

        (OrderedFloat(weight), total)
    }
}

/* Histogram Rendering */

pub fn render(bins: &[u64]) -> String {
    if bins.is_empty() {
        return String::new();
    }

    let max = bins
        .iter()
        .max()
        .copied()
        .unwrap_or(1)
        .max(1);

    bins.iter()
        .map(|&c| bar_char(c, max))
        .collect()
}

fn bar_char(count: u64, max: u64) -> char {
    if count == 0 {
        return ' ';
    }
    let ratio = count as f64 / max as f64;
    let idx = (ratio * 8.0).ceil() as usize - 1;
    HISTOGRAM_BARS[idx.min(7)]
}

/* TESTS */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btreemap_record_and_centroid_methods() {
        let mut sketch = TickSketch::new(100);

        sketch.record(10000);
        sketch.record(10000);
        sketch.record(20000);
        sketch.record(30000);

        assert_eq!(sketch.centroids.len(), 3);
        assert_eq!(sketch.min_ns(), 10000);
        assert_eq!(sketch.max_ns(), 30000);

        let values = sketch.centroid_values();
        let counts = sketch.centroid_counts();

        assert_eq!(values.len(), 3);
        assert_eq!(counts.len(), 3);
        assert_eq!(values, vec![10000, 20000, 30000]);
        assert_eq!(counts, vec![2, 1, 1]);
    }

    #[test]
    fn test_merge_closest_btreemap() {
        let mut sketch = TickSketch::new(3);

        sketch.record(10000);
        sketch.record(20000);
        sketch.record(30000);
        sketch.record(40000);

        assert_eq!(sketch.centroids.len(), 3);
    }

    #[test]
    fn test_high_frequency_updates() {
        let mut sketch = TickSketch::new(1000);

        for i in 0..10000 {
            sketch.record((i % 100) * 1000);
        }

        assert!(sketch.centroids.len() <= 1000);
        assert!(sketch.min_ns() < sketch.max_ns());
    }
}
