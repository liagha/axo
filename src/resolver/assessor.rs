use {
    crate::{
        data::{Float, Str},
        internal::operation::Range,
        resolver::{
            Resolvable, HintKind, ResolveHint,
            ErrorKind, ResolveError
        },
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{Token, TokenKind},
    },
    matchete::{string::*, Assessment, Assessor, Resemblance, Resembler, Scheme},
};

pub struct Aligner<'aligner> {
    pub assessor: Assessor<'aligner, String, String, ()>,
    pub perfection: Range<f64>,
    pub suggestion: Range<f64>,
}

impl Aligner<'_> {
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
            perfection: 0.75..1.1,
            suggestion: 0.3..0.75,
        }
    }
}

impl<'aligner> Resembler<Str<'aligner>, Str<'aligner>, ()> for Aligner<'aligner> {
    fn assessment(&mut self, query: &Str, candidate: &Str) -> Assessment<()> {
        self.assessor
            .assessment(&query.to_string(), &candidate.to_string())
    }
}

impl<'aligner> Resembler<Token<'aligner>, Token<'aligner>, ()> for Aligner<'aligner> {
    fn assessment(
        &mut self,
        query: &Token<'aligner>,
        candidate: &Token<'aligner>,
    ) -> Assessment<()> {
        match (&query.kind, &candidate.kind) {
            (TokenKind::Identifier(query), TokenKind::Identifier(candidate)) => {
                self.assessment(query, candidate)
            }
            (TokenKind::Float(query), TokenKind::Float(candidate)) => Assessment {
                resemblance: Resemblance::from(Float::abs(*query - *candidate).0),
                errors: Vec::new(),
            },
            (TokenKind::Integer(query), TokenKind::Integer(candidate)) => Assessment {
                resemblance: Resemblance::from(i128::abs(*query - *candidate) as f64),
                errors: Vec::new(),
            },
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
            (TokenKind::Comment(_), TokenKind::Comment(_)) => Assessment {
                resemblance: Resemblance::Disparity,
                errors: Vec::new(),
            },
            _ => Assessment {
                resemblance: Resemblance::Disparity,
                errors: Vec::new(),
            },
        }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>>
for Aligner<'aligner>
{
    fn assessment(
        &mut self,
        query: &Element<'aligner>,
        candidate: &Symbol<'aligner>,
    ) -> Assessment<ResolveError<'aligner>> {
        if let (Some(query), Some(candidate)) = (query.brand(), candidate.brand()) {
            let assessment = self.assessment(query, &candidate);

            if assessment.errors.is_empty() {
                let score = assessment.resemblance.to_f64();

                if self.perfection.contains(&score) {
                    Assessment {
                        resemblance: assessment.resemblance,
                        errors: Vec::new(),
                    }
                } else if self.suggestion.contains(&score) {
                    let how = "are similar".to_string();

                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![ResolveError {
                            kind: ErrorKind::UndefinedSymbol {
                                query: query.clone(),
                            },
                            span: query.span.clone(),
                            hints: vec![ResolveHint::new(HintKind::SimilarBrand {
                                candidate: candidate.clone(),
                                how,
                            })],
                        }],
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![ResolveError {
                            kind: ErrorKind::UndefinedSymbol {
                                query: query.clone(),
                            },
                            span: query.span.clone(),
                            hints: Vec::new(),
                        }],
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
}

impl Affinity {
    pub fn new() -> Self {
        Affinity {
            shaping: 0.5,
            binding: 0.5,
        }
    }
}

impl<'aligner> Resembler<Element<'aligner>, Symbol<'aligner>, ResolveError<'aligner>> for Affinity {
    fn assessment(
        &mut self,
        query: &Element<'aligner>,
        candidate: &Symbol<'aligner>,
    ) -> Assessment<ResolveError<'aligner>> {
        let mut score = 0.0;

        match (query.kind.clone(), candidate.kind.clone()) {
            (
                ElementKind::Literal(Token {
                                         kind: TokenKind::Identifier(_),
                                         ..
                                     }),
                _,
            ) => {
                score += self.shaping;
                score += self.binding;
            }

            (ElementKind::Invoke(invoke), SymbolKind::Function(function)) => {
                score += self.shaping;

                let mut errors = Vec::new();

                if invoke.members.len() > function.members.len() {
                    let diff = (invoke.members.len() - function.members.len()) as f64;
                    score += self.binding * (1.0 - (diff / invoke.members.len() as f64));

                    for member in invoke.members[function.members.len()..].iter() {
                        if let Some(brand) = member.brand() {
                            let target = invoke.target.brand().cloned().unwrap_or_else(|| brand.clone());
                            errors.push(
                                ResolveError::new(
                                    ErrorKind::MissingMember {
                                        target,
                                        member: brand.clone(),
                                    },
                                    member.span.clone(),
                                )
                            );
                        }
                    }
                } else if invoke.members.len() < function.members.len() {
                    let diff = (function.members.len() - invoke.members.len()) as f64;
                    score += self.binding * (1.0 - (diff / function.members.len() as f64));

                    for member in function.members[invoke.members.len()..].iter() {
                        if let Some(brand) = member.brand() {
                            let target = function.target.brand().cloned().unwrap_or_else(|| brand.clone());
                            errors.push(
                                ResolveError::new(
                                    ErrorKind::UndefinedMember {
                                        target,
                                        member: brand.clone(),
                                    },
                                    member.span.clone(),
                                )
                            );
                        }
                    }
                } else {
                    score += self.binding;
                }

                return Assessment {
                    resemblance: Resemblance::from(score),
                    errors,
                }
            }

            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                score += self.shaping;

                let candidates = structure
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.brand())
                    .collect::<Vec<_>>();

                let members = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.brand(),
                        _ => member.brand(),
                    })
                    .collect::<Vec<_>>();

                if candidates == members {
                    score += self.binding;
                } else {
                    let mut unique = Vec::new();
                    let mut matching = 0;

                    for member in &members {
                        if candidates.contains(member) && !unique.contains(member) {
                            matching += 1;
                            unique.push(*member);
                        }
                    }

                    let expected = candidates.len().max(members.len());
                    let ratio = if expected > 0 {
                        matching as f64 / expected as f64
                    } else {
                        1.0
                    };

                    score += self.binding * ratio;

                    let mut errors = Vec::new();

                    for member in &members {
                        if !candidates.contains(member) {
                            if let Some(target) = structure.target.brand() {
                                errors.push(ResolveError {
                                    kind: ErrorKind::UndefinedMember {
                                        target: target.clone(),
                                        member: (*member).clone(),
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                })
                            }
                        }
                    }

                    for candidate in candidates {
                        if !members.contains(&candidate) {
                            if let Some(target) = structure.target.brand() {
                                errors.push(ResolveError {
                                    kind: ErrorKind::MissingMember {
                                        target: target.clone(),
                                        member: candidate.clone(),
                                    },
                                    span: query.span.clone(),
                                    hints: Vec::new(),
                                })
                            }
                        }
                    }

                    return Assessment {
                        resemblance: Resemblance::from(score),
                        errors,
                    };
                }
            }
            (ElementKind::Construct(construct), SymbolKind::Union(union)) => {
                score += self.shaping;

                let candidates = union
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.brand())
                    .collect::<Vec<_>>();

                let members = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.brand(),
                        _ => member.brand(),
                    })
                    .collect::<Vec<_>>();

                let mut errors = Vec::new();

                if members.len() > 1 {
                    if let Some(target) = union.target.brand() {
                        let values = members.iter().map(|item| (*item).clone()).collect();
                        errors.push(ResolveError {
                            kind: ErrorKind::ExcessiveUnionMembers {
                                target: target.clone(),
                                members: values,
                            },
                            span: query.span.clone(),
                            hints: Vec::new(),
                        });
                    }
                } else if members.is_empty() {
                    if let Some(target) = union.target.brand() {
                        errors.push(ResolveError {
                            kind: ErrorKind::MissingMember {
                                target: target.clone(),
                                member: target.clone(),
                            },
                            span: query.span.clone(),
                            hints: Vec::new(),
                        });
                    }
                }

                for member in &members {
                    if !candidates.contains(member) {
                        if let Some(target) = union.target.brand() {
                            errors.push(ResolveError {
                                kind: ErrorKind::UndefinedMember {
                                    target: target.clone(),
                                    member: (*member).clone(),
                                },
                                span: query.span.clone(),
                                hints: Vec::new(),
                            });
                        }
                    }
                }

                if errors.is_empty() && members.len() == 1 {
                    score += self.binding;
                } else {
                    let mut unique = Vec::new();
                    let mut matching = 0;

                    for member in &members {
                        if candidates.contains(member) && !unique.contains(member) {
                            matching += 1;
                            unique.push(*member);
                        }
                    }

                    let ratio = if !members.is_empty() {
                        (matching as f64 / members.len() as f64) * (1.0 / members.len() as f64)
                    } else {
                        0.0
                    };
                    score += self.binding * ratio;
                }

                return Assessment {
                    resemblance: Resemblance::from(score),
                    errors,
                };
            }

            _ => {}
        };

        Assessment {
            resemblance: Resemblance::from(score),
            errors: Vec::new(),
        }
    }
}
