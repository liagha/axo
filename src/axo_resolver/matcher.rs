use {
    super::{
        brand::Branded,
        hint::ResolveHint,
    },
    crate::{
        axo_scanner::{
            Token, TokenKind,
        },
        axo_parser::{Element, ElementKind, Symbol, SymbolKind},
        axo_error::Hint,
        axo_resolver::{
            ResolveError,
            error::{
                ErrorKind,
            },
        },

        float::FloatLiteral,
    },
    core::ops::Range,
    matchete::{
        Scheme,
        Resembler, Resemblance, Assessor,
        string::*,
    },
};

#[derive(Debug)]
pub struct Aligner {
    pub assessor: Assessor<String, String, ()>,
    pub perfection: Range<f64>,
    pub suggestion: Range<f64>,
}

impl Aligner {
    pub fn new() -> Self {
        Aligner { assessor: aligner(), perfection: 0.95..1.1, suggestion: 0.2..0.95 }
    }
}

impl Resembler<String, String, ()> for Aligner {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        self.assessor.resemblance(query, candidate)
    }
}

pub fn aligner() -> Assessor<String, String, ()> {
    Assessor::new()
        .dimension(Exact, 0.02)
        .dimension(Relaxed, 0.02)
        .dimension(Prefix, 0.15)
        .dimension(Suffix, 0.14)
        .dimension(Contains, 0.13)
        .dimension(Keyboard::default(), 0.10)
        .dimension(Words::default(), 0.10)
        .dimension(Phonetic::default(), 0.07)
        .dimension(Sequential::default(), 0.06)
        .dimension(Jaro::default(), 0.05)
        .dimension(Cosine::default(), 0.04)
        .scheme(Scheme::Additive)
}

impl Resembler<Token, Token, ()> for Aligner {
    fn resemblance(&mut self, query: &Token, candidate: &Token) -> Result<Resemblance, ()> {
        match (&query.kind, &candidate.kind) {
            (TokenKind::Identifier(query), TokenKind::Identifier(candidate)) => {
                self.resemblance(query, candidate)
            }
            (TokenKind::Float(query), TokenKind::Float(candidate)) => {
                Ok(Resemblance::from(FloatLiteral::abs(*query - *candidate).0))
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

impl Resembler<Element, Symbol, ResolveError> for Aligner {
    fn resemblance(&mut self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        if let (Some(query), Some(candidate)) = (query.brand(), candidate.brand()) {
            match self.resemblance(&query, &candidate) {
                Ok(resemblance) => {
                    if self.perfection.contains(&resemblance.to_f64()) {
                        Ok(resemblance)
                    } else if self.suggestion.contains(&resemblance.to_f64()) {
                        let effective = self.assessor.dominant().unwrap().resembler.clone();
                        let message = ResolveHint::SimilarBrand { candidate, effective };

                        Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: vec![Hint {
                                    message,
                                    action: vec![],
                                }],
                                note: None,
                            }
                        )
                    } else {
                        Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: vec![],
                                note: None,
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

impl Resembler<Element, Symbol, ResolveError> for Affinity {
    fn resemblance(&mut self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.kind.clone()) {
            (ElementKind::Identifier(_), _) => {
                score += self.shaping;

                score += self.binding;
            }

            (ElementKind::Invoke(invoke), SymbolKind::Method(function)) => {
                score += self.shaping;

                if invoke.get_arguments().len() == function.get_parameters().len() {
                    score += self.binding;
                } else {
                    return Err(
                        ResolveError {
                            kind: ErrorKind::BindMismatch { candidate: function.get_name().brand().unwrap() },
                            span: query.span.clone(),
                            hints: vec![],
                            note: None,
                        }
                    )
                }
            }
            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                score += self.shaping;

                if construct.get_fields().len() == structure.get_fields().len() {
                    score += self.binding;
                } else {
                    return Err(
                        ResolveError {
                            kind: ErrorKind::BindMismatch { candidate: structure.get_name().brand().unwrap() },
                            span: query.span.clone(),
                            hints: vec![],
                            note: None,
                        }
                    )
                }
            }
            _ => {}
        };

        Ok(Resemblance::from(score))
    }
}

pub fn symbol_matcher() -> Assessor<Element, Symbol, ResolveError> {
    Assessor::<Element, Symbol, ResolveError>::new()
        .floor(0.65)
        .dimension(Aligner::new(), 0.75)
        .dimension(Affinity::new(), 0.25)
        .scheme(Scheme::Multiplicative)
}