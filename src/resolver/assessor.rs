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
        Assessment, Scheme,
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
            perfection: 0.75..1.1, suggestion: 0.3..0.75,
        }
    }
}

impl<'aligner> Resembler<Str<'aligner>, Str<'aligner>, ()> for Aligner<'aligner> {
    fn assessment(&mut self, query: &Str, candidate: &Str) -> Assessment<()> {
        self.assessor.assessment(&query.to_string(), &candidate.to_string())
    }
}

impl<'aligner> Resembler<Token<'aligner>, Token<'aligner>, ()> for Aligner<'aligner> {
    fn assessment(&mut self, query: &Token<'aligner>, candidate: &Token<'aligner>) -> Assessment<()> {
        match (&query.kind, &candidate.kind) {
            (TokenKind::Identifier(query), TokenKind::Identifier(candidate)) => {
                self.assessment(query, candidate)
            }
            (TokenKind::Float(query), TokenKind::Float(candidate)) => {
                Assessment {
                    resemblance: Resemblance::from(Float::abs(*query - *candidate).0),
                    errors: Vec::new(),
                }
            }
            (TokenKind::Integer(query), TokenKind::Integer(candidate)) => {
                Assessment {
                    resemblance: Resemblance::from(i128::abs(*query - *candidate) as f64),
                    errors: Vec::new(),
                }
            }
            (TokenKind::Boolean(query), TokenKind::Boolean(candidate)) => {
                if query == candidate {
                    Assessment {
                        resemblance: Resemblance::Perfect,
                        errors: Vec::new(),
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: Vec::new(),
                    }
                }
            }
            (TokenKind::String(query), TokenKind::String(candidate)) => {
                self.assessment(query, candidate)
            }
            (TokenKind::Character(query), TokenKind::Character(candidate)) => {
                if query == candidate {
                    Assessment {
                        resemblance: Resemblance::Perfect,
                        errors: Vec::new(),
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: Vec::new(),
                    }

                }
            }
            (TokenKind::Operator(query), TokenKind::Operator(candidate)) => {
                if query == candidate {
                    Assessment {
                        resemblance: Resemblance::Perfect,
                        errors: Vec::new(),
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: Vec::new(),
                    }
                }
            }
            (TokenKind::Punctuation(query), TokenKind::Punctuation(candidate)) => {
                if query == candidate {
                    Assessment {
                        resemblance: Resemblance::Perfect,
                        errors: Vec::new(),
                    }

                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: Vec::new(),
                    }

                }
            }
            (TokenKind::Comment(_), TokenKind::Comment(_)) => {
                Assessment {
                    resemblance: Resemblance::Disparity,
                    errors: Vec::new(),
                }
            }
            _ => {
                Assessment {
                    resemblance: Resemblance::Disparity,
                    errors: Vec::new(),
                }
            }
        }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Aligner<'aligner> {
    fn assessment(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Assessment<ResolveError<'aligner>> {
        if let (Some(query), Some(candidate)) = (query.brand(), candidate.brand()) {
            let assessment = self.assessment(&query, &candidate);

            if assessment.errors.is_empty() {
                let score = assessment.resemblance.to_f64();

                if self.perfection.contains(&score) {
                    Assessment {
                        resemblance: assessment.resemblance,
                        errors: Vec::new(),
                    }
                } else if self.suggestion.contains(&score) {
                    let dominant = self.assessor.dominant();
                    let how = if let Some(d) = dominant {
                        format!("{:?}", d.resembler)
                    } else {
                        "are similar".to_string()
                    };

                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: Vec::new(),
                            }
                        ],
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol { query: query.clone() },
                                span: query.span.clone(),
                                hints: Vec::new(),
                            }
                        ],
                    }
                }
            } else {
                Assessment {
                    resemblance: Resemblance::Disparity,
                    errors: Vec::new(),
                }
            }
        } else {
            Assessment {
                resemblance: Resemblance::Disparity,
                errors: Vec::new(),
            }
        }
    }
}

#[derive(Debug)]
pub struct Affinity {
    shaping: f64,
    binding: f64,
    base: f64,
}

impl Affinity {
    pub fn new() -> Self {
        Affinity { shaping: 0.5, binding: 0.5, base: 1.0 }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Affinity {
    fn assessment(&mut self, query: &Element<'aligner>, candidate: &Symbol<'aligner>) -> Assessment<ResolveError<'aligner>> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.kind.clone()) {
            (ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }), _) => {
                score += self.shaping;
                score += self.binding;
            }

            (ElementKind::Invoke(invoke), SymbolKind::Method(method)) => {
                score += self.shaping;

                if method.variadic {
                    score += self.binding;
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
                        let matching = members.iter()
                            .filter(|member| candidates.contains(member))
                            .count();

                        let expected = candidates.len().max(members.len());
                        let ratio = if expected > 0 {
                            matching as f64 / expected as f64
                        } else {
                            1.0
                        };

                        score += self.binding * (ratio + self.base).min(1.0);

                        let mut errors = Vec::new();

                        for member in &members {
                            if !candidates.contains(&member) {
                                errors.push(
                                    ResolveError {
                                        kind: ErrorKind::UndefinedMember {
                                            target: method.target.brand().unwrap(),
                                            member: member.clone(),
                                        },
                                        span: query.span.clone(),
                                        hints: Vec::new(),
                                    }
                                )
                            }
                        }

                        for candidate in &candidates {
                            if !members.contains(&candidate) {
                                errors.push(
                                    ResolveError {
                                        kind: ErrorKind::MissingMember {
                                            target: method.target.brand().unwrap(),
                                            member: candidate.clone(),
                                        },
                                        span: query.span.clone(),
                                        hints: Vec::new(),
                                    }
                                )
                            }
                        }

                        return Assessment {
                            resemblance: Resemblance::from(score),
                            errors,
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
                    let matching = members.iter()
                        .filter(|member| candidates.contains(member))
                        .count();

                    let expected = candidates.len().max(members.len());
                    let ratio = if expected > 0 {
                        matching as f64 / expected as f64
                    } else {
                        1.0
                    };

                    score += self.binding * (ratio + self.base).min(1.0);

                    let mut errors = Vec::new();

                    for member in &members {
                        if !candidates.contains(&member) {
                            errors.push(
                                ResolveError {
                                    kind: ErrorKind::UndefinedMember {
                                        target: structure.target.brand().unwrap(),
                                        member: member.clone(),
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                }
                            )
                        }
                    }

                    for candidate in &candidates {
                        if !members.contains(&candidate) {
                            errors.push(
                                ResolveError {
                                    kind: ErrorKind::MissingMember {
                                        target: structure.target.brand().unwrap(),
                                        member: candidate.clone(),
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                }
                            )
                        }
                    }

                    return Assessment {
                        resemblance: Resemblance::from(score),
                        errors,
                    }
                }
            }
            _ => {}
        };

        Assessment {
            resemblance: Resemblance::from(score),
            errors: Vec::new(),
        }
    }
}