use {
    super::{ErrorKind, ResolveError, ResolveHint, HintKind},
    crate::{
        data::{Float, Str},
        internal::operation::Range,
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
            (TokenKind::Identifier(left), TokenKind::Identifier(right)) => {
                self.assessment(left, right)
            }
            (TokenKind::Float(left), TokenKind::Float(right)) => Assessment {
                resemblance: Resemblance::from(Float::abs(*left - *right).0),
                errors: Vec::new(),
            },
            (TokenKind::Integer(left), TokenKind::Integer(right)) => Assessment {
                resemblance: Resemblance::from(i128::abs(*left - *right) as f64),
                errors: Vec::new(),
            },
            (TokenKind::Boolean(left), TokenKind::Boolean(right)) => {
                if left == right {
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
            (TokenKind::String(left), TokenKind::String(right)) => {
                self.assessment(left, right)
            }
            (TokenKind::Character(left), TokenKind::Character(right)) => {
                if left == right {
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
            (TokenKind::Operator(left), TokenKind::Operator(right)) => {
                if left == right {
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
            (TokenKind::Punctuation(left), TokenKind::Punctuation(right)) => {
                if left == right {
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
        if let (Some(left), Some(right)) = (query.brand(), candidate.brand()) {
            let assessment = self.assessment(left, &right);

            if assessment.errors.is_empty() {
                let score = assessment.resemblance.to_f64();

                if self.perfection.contains(&score) {
                    Assessment {
                        resemblance: assessment.resemblance,
                        errors: Vec::new(),
                    }
                } else if self.suggestion.contains(&score) {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![ResolveError {
                            kind: ErrorKind::UndefinedSymbol {
                                query: left.clone(),
                            },
                            span: left.span.clone(),
                            hints: vec![ResolveHint::new(HintKind::SimilarBrand {
                                candidate: right.clone(),
                                how: "are similar".to_string(),
                            })],
                        }],
                    }
                } else {
                    Assessment {
                        resemblance: Resemblance::Disparity,
                        errors: vec![ResolveError {
                            kind: ErrorKind::UndefinedSymbol {
                                query: left.clone(),
                            },
                            span: left.span.clone(),
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

                let diff = (invoke.members.len() as f64 - function.members.len() as f64).abs();
                let extent = invoke.members.len().max(function.members.len()) as f64;

                if extent > 0.0 {
                    score += self.binding * (1.0 - (diff / extent));
                } else {
                    score += self.binding;
                }
            }

            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                score += self.shaping;

                let form: Vec<_> = structure
                    .members
                    .iter()
                    .filter_map(|member| member.brand())
                    .collect();

                let input: Vec<_> = construct
                    .members
                    .iter()
                    .filter_map(|member| member.brand())
                    .collect();

                let matching = input.iter().filter(|item| form.contains(item)).count();
                let extent = form.len().max(input.len()) as f64;

                if extent > 0.0 {
                    score += self.binding * (matching as f64 / extent);
                } else {
                    score += self.binding;
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
