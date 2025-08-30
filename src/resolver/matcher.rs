use {
    super::{
        ResolveError,
        ErrorKind,
    },
    crate::{
        scanner::{
            Token, TokenKind,
        },
        parser::{
            Element, ElementKind,
            Symbol, Symbolic,
        },
        schema::{Method, Structure},
        data::{
            Float, Str,
        },
        internal::operation::Range,
    },
    matchete::{
        Scheme,
        Resembler, Resemblance, Assessor,
        string::*,
    },
};
use crate::resolver::{HintKind, ResolveHint};

#[derive(Debug)]
pub struct Aligner<'aligner> {
    pub assessor: Assessor<'aligner, String, String, ()>,
    pub perfection: Range<f64>,
    pub suggestion: Range<f64>,
}

impl<'aligner> Aligner<'aligner> {
    pub fn new() -> Self {
        Aligner { assessor: aligner(), perfection: 0.85..1.1, suggestion: 0.3..0.85, }
    }
}

impl<'aligner> Resembler<Str<'aligner>, Str<'aligner>, ()> for Aligner<'aligner> {
    fn resemblance(&mut self, query: &Str, candidate: &Str) -> Result<Resemblance, ()> {
        self.assessor.resemblance(&query.to_string(), &candidate.to_string())
    }
}

pub fn aligner<'aligner>() -> Assessor<'aligner, String, String, ()> {
    Assessor::new()
        .dimension(Box::leak(Box::new(Exact)), 0.05)
        .dimension(Box::leak(Box::new(Relaxed)), 0.05)
        .dimension(Box::leak(Box::new(Prefix)), 0.15)
        .dimension(Box::leak(Box::new(Suffix)), 0.1)
        .dimension(Box::leak(Box::new(Contains)), 0.1)
        .dimension(Box::leak(Box::new(Keyboard::default())), 0.1)
        .dimension(Box::leak(Box::new(Words::default())), 0.1)
        .dimension(Box::leak(Box::new(Phonetic::default())), 0.1)
        .dimension(Box::leak(Box::new(Sequential::default())), 0.05)
        .dimension(Box::leak(Box::new(Jaro::default())), 0.1)
        .dimension(Box::leak(Box::new(Cosine::default())), 0.1)
        .scheme(Scheme::Additive)
}

impl<'aligner> Resembler<Token<'aligner>, Token<'aligner>, ()> for Aligner<'aligner> {
    fn resemblance(&mut self, query: &Token<'aligner>, candidate: &Token<'aligner>) -> Result<Resemblance, ()> {
        match (&query.kind, &candidate.kind) {
            (TokenKind::Identifier(query), TokenKind::Identifier(candidate)) => {
                self.resemblance(query, candidate)
            }
            (TokenKind::Float(query), TokenKind::Float(candidate)) => {
                Ok(Resemblance::from(Float::abs(*query - *candidate).0))
            }
            (TokenKind::Integer(query), TokenKind::Integer(candidate)) => {
                Ok(Resemblance::from(i128::abs(*query - *candidate) as f64))
            }
            (TokenKind::Boolean(query), TokenKind::Boolean(candidate)) => {
                if query == candidate {
                    Ok(Resemblance::Perfect)
                } else {
                    Ok(Resemblance::Disparity)
                }
            }
            (TokenKind::String(query), TokenKind::String(candidate)) => {
                self.resemblance(query, candidate)
            }
            (TokenKind::Character(query), TokenKind::Character(candidate)) => {
                if query == candidate {
                    Ok(Resemblance::Perfect)
                } else {
                    Ok(Resemblance::Disparity)
                }
            }
            (TokenKind::Operator(query), TokenKind::Operator(candidate)) => {
                if query == candidate {
                    Ok(Resemblance::Perfect)
                } else {
                    Ok(Resemblance::Disparity)
                }
            }
            (TokenKind::Punctuation(query), TokenKind::Punctuation(candidate)) => {
                if query == candidate {
                    Ok(Resemblance::Perfect)
                } else {
                    Ok(Resemblance::Disparity)
                }
            }
            (TokenKind::Comment(_), TokenKind::Comment(_)) => {
                Ok(Resemblance::Disparity)
            }
            _ => {
                Ok(Resemblance::Disparity)
            }
        }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Aligner<'aligner> {
    fn resemblance(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Result<Resemblance, ResolveError<'aligner>> {
        if let (Some(query), Some(candidate)) = (query.brand(), candidate.brand()) {
            match self.resemblance(&query, &candidate) {
                Ok(resemblance) => {
                    let score = resemblance.to_f64();

                    if self.perfection.contains(&score) {
                        Ok(resemblance)
                    } else if self.suggestion.contains(&score) {
                        let dominant = self.assessor.dominant();
                        let how = if let Some(d) = dominant {
                            format!("{:?}", d.resembler)
                        } else {
                            "are similar".to_string()
                        };
                        Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: vec![ResolveHint::new(HintKind::SimilarBrand { candidate: candidate.clone(), how })],
                            }
                        )
                    } else {
                        Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: vec![],
                            }
                        )
                    }
                }
                Err(_) => {
                    Ok(Resemblance::Disparity)
                },
            }
        } else {
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug)]
struct Affinity {
    shaping: f64,
    binding: f64,
}

impl Affinity {
    fn new() -> Self {
        Affinity { shaping: 0.75, binding: 0.25 }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Affinity {
    fn resemblance(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Result<Resemblance, ResolveError<'aligner>> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.clone()) {
            (ElementKind::Identifier(_), _) => {
                score += self.shaping;

                score += self.binding;
            }

            (ElementKind::Invoke(invoke), candidate) => {
                if let Symbolic::Method(method) = candidate.value {
                    score += self.shaping;

                    if invoke.get_arguments().len() == method.get_parameters().len() {
                        score += self.binding;
                    } else {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::BindMismatch { candidate: method.get_target().brand().unwrap() },
                                span: query.span.clone(),
                                hints: vec![],
                            }
                        )
                    }
                }
            }
            (ElementKind::Construct(construct), candidate) => {
                if let Symbolic::Structure(structure) = candidate.value {
                    score += self.shaping;

                    if construct.get_fields().len() == structure.get_fields().len() {
                        score += self.binding;
                    } else {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::BindMismatch { candidate: structure.get_target().brand().unwrap() },
                                span: query.span.clone(),
                                hints: vec![],
                            }
                        )
                    }
                }
            }
            _ => {}
        };

        Ok(Resemblance::from(score))
    }
}

pub fn symbol_matcher<'matcher>() -> Assessor<'matcher, Element<'matcher>, Symbol<'matcher>, ResolveError<'matcher>> {
    Assessor::new()
        .floor(0.5)
        .dimension(Box::leak(Box::new(Aligner::new())), 0.75)
        .dimension(Box::leak(Box::new(Affinity::new())), 0.25)
        .scheme(Scheme::Multiplicative)
}