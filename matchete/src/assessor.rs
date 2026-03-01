#[derive(Clone, PartialEq)]
pub enum Resemblance {
    Perfect,
    Partial(f64),
    Disparity,
}

impl From<f64> for Resemblance {
    fn from(f: f64) -> Self {
        if f == 0.0 {
            Resemblance::Disparity
        } else if f == 1.0 {
            Resemblance::Perfect
        } else {
            Resemblance::Partial(f)
        }
    }
}

impl From<Resemblance> for f64 {
    fn from(r: Resemblance) -> Self {
        match r {
            Resemblance::Disparity => 0.0,
            Resemblance::Perfect => 1.0,
            Resemblance::Partial(f) => f,
        }
    }
}

impl Resemblance {
    pub fn to_f64(&self) -> f64 {
        match self {
            Resemblance::Disparity => 0.0,
            Resemblance::Perfect => 1.0,
            Resemblance::Partial(f) => *f,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Assessment<Error> {
    pub resemblance: Resemblance,
    pub errors: Vec<Error>,
}

#[derive(Clone, PartialEq)]
pub enum Scheme {
    Additive,
    Multiplicative,
    Minimum,
    Maximum,
    Threshold,
    Harmonic,
}

impl Default for Scheme {
    fn default() -> Self {
        Scheme::Additive
    }
}

pub trait Resembler<Query, Candidate, Error>: Send + Sync {
    fn assessment(&mut self, query: &Query, candidate: &Candidate) -> Assessment<Error>;
}

pub struct Dimension<'dimension, Query, Candidate, Error> {
    pub resembler: &'dimension mut dyn Resembler<Query, Candidate, Error>,
    pub weight: f64,
    pub assessment: Assessment<Error>,
    pub contribution: f64,
}

impl<'dimension, Query, Candidate, Error> Dimension<'dimension, Query, Candidate, Error> {
    pub fn new<R: Resembler<Query, Candidate, Error> + 'dimension>(resembler: &'dimension mut R, weight: f64) -> Self {
        Self {
            resembler,
            weight,
            assessment: Assessment { resemblance: Resemblance::Disparity, errors: vec![] },
            contribution: 0.0,
        }
    }

    pub fn assess(&mut self, query: &Query, candidate: &Candidate) {
        self.assessment = self.resembler.assessment(query, candidate);
        self.contribution = if self.assessment.errors.is_empty() {
            self.assessment.resemblance.to_f64() * self.weight
        } else {
            0.0
        };
    }
}

pub struct Assessor<'assessor, Query, Candidate, Error> {
    pub dimensions: Vec<Dimension<'assessor, Query, Candidate, Error>>,
    pub floor: f64,
    pub scheme: Scheme,
    pub errors: Vec<Error>,
}

impl<'assessor, Query, Candidate, Error> Assessor<'assessor, Query, Candidate, Error>
where
    Query: Clone,
    Candidate: Clone,
    Error: Clone,
{
    pub fn new() -> Self {
        Self {
            dimensions: Vec::new(),
            floor: 0.4,
            scheme: Scheme::default(),
            errors: Vec::new(),
        }
    }

    pub fn floor(mut self, floor: f64) -> Self {
        self.floor = floor;
        self
    }

    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = scheme;
        self
    }

    pub fn dimension<R: Resembler<Query, Candidate, Error>>(
        mut self,
        resembler: &'assessor mut R,
        weight: f64,
    ) -> Self {
        self.dimensions.push(Dimension::new(resembler, weight));
        self
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn calculate_resemblance(&self, dimensions: &[Dimension<Query, Candidate, Error>]) -> f64 {
        let successful_dimensions: Vec<_> = dimensions
            .iter()
            .filter(|d| d.assessment.errors.is_empty())
            .collect();

        if successful_dimensions.is_empty() {
            return 0.0;
        }

        match self.scheme {
            Scheme::Additive => {
                let total_contribution: f64 = successful_dimensions.iter().map(|d| d.contribution).sum();
                let total_weight: f64 = successful_dimensions.iter().map(|d| d.weight).sum();
                if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 }
            }
            Scheme::Multiplicative => {
                let product: f64 = successful_dimensions.iter()
                    .map(|d| d.assessment.resemblance.to_f64().powf(d.weight))
                    .product();
                let total_weight: f64 = successful_dimensions.iter().map(|d| d.weight).sum();
                if total_weight > 0.0 { product.powf(1.0 / total_weight) } else { 0.0 }
            }
            Scheme::Minimum => {
                successful_dimensions.iter()
                    .map(|d| d.assessment.resemblance.to_f64())
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
            }
            Scheme::Maximum => {
                successful_dimensions.iter()
                    .map(|d| d.assessment.resemblance.to_f64())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
            }
            Scheme::Threshold => {
                let threshold = 0.5;
                if successful_dimensions.iter().all(|d| d.assessment.resemblance.to_f64() >= threshold) {
                    let total_contribution: f64 = successful_dimensions.iter().map(|d| d.contribution).sum();
                    let total_weight: f64 = successful_dimensions.iter().map(|d| d.weight).sum();
                    if total_weight > 0.0 { total_contribution / total_weight } else { 0.0 }
                } else {
                    0.0
                }
            }
            Scheme::Harmonic => {
                let sum_reciprocals: f64 = successful_dimensions.iter()
                    .map(|d| d.weight / d.assessment.resemblance.to_f64())
                    .sum();
                let total_weight: f64 = successful_dimensions.iter().map(|d| d.weight).sum();
                if sum_reciprocals.is_finite() && sum_reciprocals > 0.0 {
                    total_weight / sum_reciprocals
                } else {
                    0.0
                }
            }
        }
    }
}

impl<'assessor, Query, Candidate, Error> Resembler<Query, Candidate, Error> for Assessor<'assessor, Query, Candidate, Error>
where
    Query: Clone,
    Candidate: Clone,
    Error: Clone + Send + Sync,
{
    fn assessment(&mut self, query: &Query, candidate: &Candidate) -> Assessment<Error> {
        for dimension in &mut self.dimensions {
            dimension.assess(query, candidate);
        }

        let mut errors = vec![];
        for dimension in &self.dimensions {
            errors.extend(dimension.assessment.errors.clone());
        }

        let value = self.calculate_resemblance(&self.dimensions);

        let resemblance = if value >= 1.0 {
            Resemblance::Perfect
        } else if value > 0.0 {
            Resemblance::Partial(value)
        } else {
            Resemblance::Disparity
        };

        Assessment { resemblance, errors }
    }
}

impl<'assessor, Query, Candidate, Error> Assessor<'assessor, Query, Candidate, Error>
where
    Query: Clone,
    Candidate: Clone,
    Error: Clone,
{
    fn assess_candidate(&mut self, query: &Query, candidate: &Candidate) -> Option<(Resemblance, bool)> {
        self.errors.clear();

        for dimension in &mut self.dimensions {
            dimension.assess(query, candidate);
        }

        let mut errors = vec![];
        for dimension in &self.dimensions {
            errors.extend(dimension.assessment.errors.clone());
        }

        let has_errors = !errors.is_empty();

        let value = self.calculate_resemblance(&self.dimensions);

        let resemblance = value.into();

        let viable = value >= self.floor;

        if has_errors {
            self.errors = errors;
            None
        } else {
            Some((resemblance, viable))
        }
    }

    pub fn dominant(&self) -> Option<&Dimension<'assessor, Query, Candidate, Error>> {
        self.dimensions.iter()
            .filter(|d| d.assessment.errors.is_empty())
            .max_by(|a, b| a.contribution.partial_cmp(&b.contribution).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn resemblance_value(&mut self, query: &Query, candidate: &Candidate) -> Option<Resemblance> {
        self.assess_candidate(query, candidate).map(|(resemblance, _)| resemblance)
    }

    pub fn viable(&mut self, query: &Query, candidate: &Candidate) -> Option<bool> {
        self.assess_candidate(query, candidate).map(|(_, viable)| viable)
    }

    pub fn champion(&mut self, query: &Query, candidates: &[Candidate]) -> Option<Candidate> {
        let mut best_candidate = None;
        let mut best_resemblance = -1.0;

        let mut best_failed_res = -1.0;
        let mut best_failed_errors: Vec<Error> = Vec::new();

        for candidate in candidates {
            let opt = self.assess_candidate(query, candidate);
            if let Some((resemblance, viable)) = opt {
                let resemblance_val = resemblance.to_f64();

                if viable && resemblance_val > best_resemblance {
                    best_resemblance = resemblance_val;
                    best_candidate = Some(candidate.clone());
                }
            } else {
                let temp_res = self.calculate_resemblance(&self.dimensions);
                if temp_res > best_failed_res {
                    best_failed_res = temp_res;
                    best_failed_errors = self.errors.clone();
                }
            }
        }

        if best_candidate.is_some() {
            best_candidate
        } else {
            if best_failed_res > -1.0 {
                self.errors = best_failed_errors;
            }
            None
        }
    }

    pub fn shortlist(&mut self, query: &Query, candidates: &[Candidate]) -> Vec<Candidate> {
        let mut viable_candidates: Vec<(Candidate, f64)> = Vec::new();

        for candidate in candidates {
            if let Some((resemblance, viable)) = self.assess_candidate(query, candidate) {
                if viable {
                    viable_candidates.push((candidate.clone(), resemblance.to_f64()));
                }
            }
        }

        viable_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        viable_candidates.into_iter().map(|(candidate, _)| candidate).collect()
    }

    pub fn constrain(&mut self, query: &Query, candidates: &[Candidate], cap: usize) -> Vec<Candidate> {
        let mut shortlisted = self.shortlist(query, candidates);
        shortlisted.truncate(cap);
        shortlisted
    }
}