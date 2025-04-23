use crate::axo_matcher::common::SimilarityMetric;

pub struct NumericProximityMetric {
    pub normalization_factor: f64,
}

impl Default for NumericProximityMetric {
    fn default() -> Self {
        NumericProximityMetric {
            normalization_factor: 10.0,
        }
    }
}

impl<T, U> SimilarityMetric<T, U> for NumericProximityMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        let difference = (a_val - b_val).abs();
        let normalized_diff = difference / self.normalization_factor;

        (-normalized_diff).exp()
    }

    fn id(&self) -> &str {
        "NumericProximity"
    }
}

pub struct RangeProximityMetric {
    pub min_value: f64,
    pub max_value: f64,
}

impl Default for RangeProximityMetric {
    fn default() -> Self {
        RangeProximityMetric {
            min_value: 0.0,
            max_value: 100.0,
        }
    }
}

impl<T, U> SimilarityMetric<T, U> for RangeProximityMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        if a_val == b_val {
            return 1.0;
        }

        let range = self.max_value - self.min_value;
        if range <= 0.0 {
            return if a_val == b_val { 1.0 } else { 0.0 };
        }

        let max_distance = range;
        let actual_distance = (a_val - b_val).abs();

        1.0 - (actual_distance / max_distance).min(1.0)
    }

    fn id(&self) -> &str {
        "RangeProximity"
    }
}

pub struct PercentageProximityMetric;

impl<T, U> SimilarityMetric<T, U> for PercentageProximityMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        if a_val == b_val {
            return 1.0;
        }

        if a_val == 0.0 || b_val == 0.0 {
            return 0.0;
        }

        let larger = a_val.max(b_val);
        let smaller = a_val.min(b_val);

        smaller / larger
    }

    fn id(&self) -> &str {
        "PercentageProximity"
    }
}

pub struct ExponentialDecayMetric {
    pub decay_factor: f64,
}

impl Default for ExponentialDecayMetric {
    fn default() -> Self {
        ExponentialDecayMetric {
            decay_factor: 0.1,
        }
    }
}

impl<T, U> SimilarityMetric<T, U> for ExponentialDecayMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        let difference = (a_val - b_val).abs();

        (-self.decay_factor * difference).exp()
    }

    fn id(&self) -> &str {
        "ExponentialDecay"
    }
}

pub struct ThresholdProximityMetric {
    pub threshold: f64,
}

impl Default for ThresholdProximityMetric {
    fn default() -> Self {
        ThresholdProximityMetric {
            threshold: 5.0,
        }
    }
}

impl<T, U> SimilarityMetric<T, U> for ThresholdProximityMetric
where
    T: Into<f64> + Copy,
    U: Into<f64> + Copy,
{
    fn calculate(&self, a: &T, b: &U) -> f64 {
        let a_val: f64 = (*a).into();
        let b_val: f64 = (*b).into();

        let difference = (a_val - b_val).abs();

        if difference <= self.threshold {
            1.0 - (difference / self.threshold)
        } else {
            0.0
        }
    }

    fn id(&self) -> &str {
        "ThresholdProximity"
    }
}