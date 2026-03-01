use {
    crate::{
        assessor::{
            Resembler, Resemblance, Assessment,
        },
    }
};

#[derive(PartialEq)]
pub struct Exact;

impl Resembler<String, String, ()> for Exact {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        let resemblance = if query == candidate {
            Resemblance::Perfect
        } else {
            Resemblance::Disparity
        };
        Assessment { resemblance, errors: vec![] }
    }
}

#[derive(PartialEq)]
pub struct Relaxed;

impl Resembler<String, String, ()> for Relaxed {
    fn assessment(&mut self, query: &String, candidate: &String) -> Assessment<()> {
        let resemblance = if query.to_lowercase() == candidate.to_lowercase() {
            Resemblance::Partial(0.95)
        } else {
            Resemblance::Disparity
        };
        Assessment { resemblance, errors: vec![] }
    }
}