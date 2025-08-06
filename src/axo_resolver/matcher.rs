use {
    super::{
        hint::ResolveHint,
    },
    crate::{
        axo_scanner::{
            Token, TokenKind,
        },
        axo_parser::{
            Element, ElementKind,
            Symbol, Symbolic,
        },
        axo_schema::{Method, Structure},
        axo_error::Hint,
        axo_resolver::{
            ResolveError,
            error::{
                ErrorKind,
            },
        },

        operations::Range,
        float::FloatLiteral,
    },
    matchete::{
        Scheme,
        Resembler, Resemblance, Assessor,
        string::*,
    },
};

#[derive(Debug)]
pub struct Aligner<'aligner> {
    pub assessor: Assessor<String, String, ()>,
    pub perfection: Range<f64>,
    pub suggestion: Range<f64>,
    pub phantom: &'aligner ()
}

impl<'aligner> Aligner<'aligner> {
    pub fn new() -> Self {
        Aligner { assessor: aligner(), perfection: 0.90..1.1, suggestion: 0.2..0.90, phantom: &() }
    }
}

impl<'aligner> Resembler<String, String, ()> for Aligner<'aligner> {
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

impl<'aligner> Resembler<Token<'aligner>, Token<'aligner>, ()> for Aligner<'aligner> {
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

impl<'aligner: 'static> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Aligner<'aligner> {
    fn resemblance(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Result<Resemblance, ResolveError<'aligner>> {
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

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Affinity {
    fn resemblance(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Result<Resemblance, ResolveError<'aligner>> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.clone()) {
            (ElementKind::Identifier(_), _) => {
                score += self.shaping;

                score += self.binding;
            }

            (ElementKind::Invoke(invoke), candidate) => {
                if let Some(method) = candidate.cast::<Method<Box<Element>, Symbol, Box<Element>, Option<Box<Element>>>>() {
                    score += self.shaping;

                    if invoke.get_arguments().len() == method.get_parameters().len() {
                        score += self.binding;
                    } else {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::BindMismatch { candidate: method.get_target().brand().unwrap() },
                                span: query.span.clone(),
                                hints: vec![],
                                note: None,
                            }
                        )
                    }
                }
            }
            (ElementKind::Construct(construct), candidate) => {
                if let Some(structure) = candidate.cast::<Structure<Box<Element>, Symbol>>() {
                    score += self.shaping;

                    if construct.get_fields().len() == structure.get_fields().len() {
                        score += self.binding;
                    } else {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::BindMismatch { candidate: structure.get_target().brand().unwrap() },
                                span: query.span.clone(),
                                hints: vec![],
                                note: None,
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

pub fn symbol_matcher<'matcher: 'static>() -> Assessor<Element<'matcher>, Symbol<'matcher>, ResolveError<'matcher>> {
    Assessor::new()
        .floor(0.65)
        .dimension(Aligner::new(), 0.75)
        .dimension(Affinity::new(), 0.25)
        .scheme(Scheme::Multiplicative)
}