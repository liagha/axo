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
            Symbol, SymbolKind,
        },
        resolver::{
            HintKind, ResolveHint,
        },
        data::{
            Float, Str
        },
        internal::{
            operation::Range,
        },
    },
    matchete::{
        Scheme,
        Resembler, Resemblance, Assessor,
        string::*,
    },
};

#[derive(Debug)]
pub struct Aligner<'aligner> {
    pub assessor: Assessor<'aligner, String, String, ()>,
    pub perfection: Range<f64>,
    pub suggestion: Range<f64>,
}

impl<'aligner> Aligner<'aligner> {
    pub fn new() -> Self {
        Aligner {
            assessor: Assessor::new()
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
                .scheme(Scheme::Additive),
            perfection: 0.85..1.1, suggestion: 0.3..0.85,
        }
    }
}

impl<'aligner> Resembler<Str<'aligner>, Str<'aligner>, ()> for Aligner<'aligner> {
    fn resemblance(&mut self, query: &Str, candidate: &Str) -> Result<Resemblance, ()> {
        self.assessor.resemblance(&query.to_string(), &candidate.to_string())
    }
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
                                hints: Vec::new(),
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
pub struct Affinity {
    shaping: f64,
    binding: f64,
}

impl Affinity {
    pub fn new() -> Self {
        Affinity { shaping: 0.75, binding: 0.25 }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Affinity {
    fn resemblance(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Result<Resemblance, ResolveError<'aligner>> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.kind.clone()) {
            (ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }), _) => {
                score += self.shaping;

                score += self.binding;
            }

            (ElementKind::Invoke(invoke), SymbolKind::Method(method)) => {
                score += self.shaping;

                if method.variadic {

                } else {
                    let candidates = method.members
                        .iter()
                        .map(|member| member.brand().unwrap())
                        .collect::<Vec<_>>();

                    let members = invoke.members
                        .iter()
                        .map(|member| member.brand().unwrap())
                        .collect::<Vec<_>>();

                    if candidates == members {
                        score += self.binding;
                    } else {
                        let undefined = members
                            .iter()
                            .cloned()
                            .filter(|member| !candidates.contains(member))
                            .collect::<Vec<_>>();

                        let missing = candidates
                            .iter()
                            .cloned()
                            .filter(|member| !members.contains(member))
                            .collect::<Vec<_>>();

                        if !missing.is_empty() {
                            return Err(
                                ResolveError {
                                    kind: ErrorKind::MissingMember {
                                        target: method.target.brand().unwrap(),
                                        members: missing,
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                }
                            )
                        } else if !undefined.is_empty() {
                            return Err(
                                ResolveError {
                                    kind: ErrorKind::UndefinedMember {
                                        target: method.target.brand().unwrap(),
                                        members: undefined,
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                }
                            )
                        }
                    }
                }
            }

            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                score += self.shaping;

                let candidates = structure.members
                    .iter()
                    .map(|member| member.brand().unwrap())
                    .collect::<Vec<_>>();

                let members = construct.members
                    .iter()
                    .map(|member| member.brand().unwrap())
                    .collect::<Vec<_>>();

                if candidates == members {
                    score += self.binding;
                } else {
                    let undefined = members
                        .iter()
                        .cloned()
                        .filter(|member| !candidates.contains(member))
                        .collect::<Vec<_>>();

                    let missing = candidates
                        .iter()
                        .cloned()
                        .filter(|member| !members.contains(member))
                        .collect::<Vec<_>>();

                    let score =
                        (undefined.len() as f64 / members.len() as f64)
                            * (missing.len() as f64 / candidates.len() as f64)
                            .min(1.0).max(0.0) * self.binding;

                    if !missing.is_empty() {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::MissingMember {
                                    target: structure.target.brand().unwrap(),
                                    members: missing,
                                },
                                span: query.span.clone(),
                                hints: Vec::new(),
                            }
                        )
                    } else if !undefined.is_empty() {
                        return Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedMember {
                                    target: structure.target.brand().unwrap(),
                                    members: undefined,
                                },
                                span: query.span.clone(),
                                hints: Vec::new(),
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