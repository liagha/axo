use matchete::Resemblance;
use {
    super::{
        brand::Branded
    },
    crate::{
        axo_scanner::Token,
        axo_parser::{Element, ElementKind, Symbol, SymbolKind},
        axo_error::Hint,
        axo_resolver::{
            ResolveError,
            error::{
                ErrorKind,
            },
        },
    },
    matchete::{
        Resembler, Assessor,
        prelude::*,
    },
};

impl Resembler<Token, Token, ()> for FullMatcher {
    fn resemblance(&self, query: &Token, candidate: &Token) -> Result<Resemblance, ()> {
        self.resemblance(&query.to_string(), &candidate.to_string())
    }
}

impl Resembler<Element, Symbol, ResolveError> for FullMatcher {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        println!("FullMatcher comparing:");
        println!("  Query: {:?}", query);
        println!("  Candidate: {:?}", candidate);

        if let (Some(query_brand), Some(candidate_brand)) = (query.brand(), candidate.brand()) {
            println!("  Query brand: {:?}", query_brand);
            println!("  Candidate brand: {:?}", candidate_brand);

            match self.resemblance(&query_brand.to_string(), &candidate_brand.to_string()) {
                Ok(resemblance) => {
                    println!("  String resemblance: {:?}", resemblance);
                    if resemblance == Resemblance::Perfect {
                        Ok(resemblance)
                    } else if resemblance.to_f64() > 0.8 {
                        Ok(resemblance)
                    } else if resemblance.to_f64() > 0.5 {

                        let message = format!("did you mean `{}`?", candidate_brand);

                        Err(
                            ResolveError {
                                kind: ErrorKind::UndefinedSymbol(query_brand.clone()),
                                span: query_brand.span.clone(),
                                hints: vec![Hint {
                                    message,
                                    action: vec![],
                                }],
                                note: None,
                            }
                        )
                    } else {
                        Ok(resemblance)
                    }
                }
                Err(_) => {
                    println!("  String comparison failed");
                    Ok(Resemblance::Disparity)
                },
            }
        } else {
            println!("  One or both brands are None");
            println!("  Query brand: {:?}", query.brand());
            println!("  Candidate brand: {:?}", candidate.brand());
            Ok(Resemblance::Disparity)
        }
    }
}

#[derive(Debug)]
pub struct SymbolType;

impl Resembler<Element, Symbol, ResolveError> for SymbolType {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let resemblance = match (&query.kind, &candidate.kind) {
            (ElementKind::Invoke { .. }, SymbolKind::Function { .. }) => 0.98,
            (ElementKind::Identifier(_), SymbolKind::Binding { .. }) => 0.95,
            (ElementKind::Identifier(_), SymbolKind::Function { .. }) => 0.9,
            (ElementKind::Identifier(_), SymbolKind::Structure { .. }) => 0.8,
            (ElementKind::Identifier(_), SymbolKind::Enumeration { .. }) => 0.75,
            (ElementKind::Constructor { .. }, SymbolKind::Structure { .. }) => 0.95,
            (ElementKind::Constructor { .. }, SymbolKind::Enumeration { .. }) => 0.9,
            _ => 0.0,
        };

        if resemblance > 0.0 && resemblance < 0.7 {
            let target_name = match query.brand() {
                Some(name) => name,
                None => return Ok(Resemblance::from(resemblance)),
            };

            let message = format!("expected {:?}, found {:?}", candidate.kind, query.kind);

            Err(ResolveError {
                kind: ErrorKind::TypeMismatch {
                    expected: format!("{:?}", candidate.kind),
                    found: format!("{:?}", query.kind),
                },
                span: target_name.span.clone(),
                note: None,
                hints: vec![Hint {
                    message,
                    action: vec![],
                }],
            })
        } else {
            Ok(Resemblance::from(resemblance))
        }
    }
}

#[derive(Debug)]
pub struct ParameterCount;

impl Resembler<Element, Symbol, ResolveError> for ParameterCount {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let resemblance = match (&query.kind, &candidate.kind) {
            (ElementKind::Constructor { fields: f1, .. }, SymbolKind::Structure { fields: f2, .. }) => {
                let constructor = f1.len();
                let structure = f2.len();
                if constructor == structure {
                    0.9
                } else if constructor < structure {
                    0.8 * (constructor as f64 / structure as f64)
                } else {
                    0.0
                }
            }
            (ElementKind::Invoke { arguments: a1, .. }, SymbolKind::Function { parameters: p1, .. }) => {
                let invoke_args = a1.len();
                let function_params = p1.len();
                if invoke_args == function_params {
                    0.9
                } else if invoke_args < function_params {
                    0.8 * (invoke_args as f64 / function_params as f64)
                } else {
                    0.0
                }
            }
            _ => 1.0, // No parameter count constraints for other combinations
        };

        // Only create errors for exact parameter mismatches, not for scoring purposes
        match (&query.kind, &candidate.kind) {
            (ElementKind::Constructor { fields: f1, .. }, SymbolKind::Structure { fields: f2, .. }) => {
                if f1.len() != f2.len() && resemblance > 0.0 && resemblance < 0.9 {
                    let target_name = match query.brand() {
                        Some(name) => name,
                        None => return Ok(Resemblance::from(resemblance)),
                    };

                    let message = format!("expected {} fields, found {}", f2.len(), f1.len());

                    return Err(ResolveError {
                        kind: ErrorKind::FieldCountMismatch {
                            expected: f2.len(),
                            found: f1.len(),
                        },
                        span: target_name.span.clone(),
                        note: None,
                        hints: vec![Hint {
                            message,
                            action: vec![],
                        }],
                    });
                }
            }
            (ElementKind::Invoke { arguments: a1, .. }, SymbolKind::Function { parameters: p1, .. }) => {
                if a1.len() != p1.len() && resemblance > 0.0 && resemblance < 0.9 {
                    let target_name = match query.brand() {
                        Some(name) => name,
                        None => return Ok(Resemblance::from(resemblance)),
                    };

                    let message = format!("expected {} parameters, found {}", p1.len(), a1.len());

                    return Err(ResolveError {
                        kind: ErrorKind::FieldCountMismatch {
                            expected: p1.len(),
                            found: a1.len(),
                        },
                        span: target_name.span.clone(),
                        note: None,
                        hints: vec![Hint {
                            message,
                            action: vec![],
                        }],
                    });
                }
            }
            _ => {}
        }

        Ok(Resemblance::from(resemblance))
    }
}

#[derive(Debug)]
pub struct ContextualRelevance {
    pub context_weight: f64,
}

impl Default for ContextualRelevance {
    fn default() -> Self {
        Self {
            context_weight: 0.85,
        }
    }
}

impl Resembler<Element, Symbol, ResolveError> for ContextualRelevance {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let resemblance = match &query.kind {
            ElementKind::Identifier(_) => {
                match &candidate.kind {
                    SymbolKind::Binding { .. } => self.context_weight,
                    SymbolKind::Function { .. } => self.context_weight - 0.1,
                    SymbolKind::Structure { .. } => self.context_weight - 0.2,
                    SymbolKind::Enumeration { .. } => self.context_weight - 0.2,
                    _ => 0.0,
                }
            }
            ElementKind::Invoke { .. } => {
                match &candidate.kind {
                    SymbolKind::Function { .. } => self.context_weight,
                    _ => 0.0,
                }
            }
            ElementKind::Constructor { .. } => {
                match &candidate.kind {
                    SymbolKind::Structure { .. } => self.context_weight,
                    SymbolKind::Enumeration { .. } => self.context_weight - 0.1,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        };

        // Only create suggestion errors for moderate similarity, not for good matches
        if resemblance > 0.4 && resemblance < 0.6 {
            if let (Some(query_brand), Some(candidate_brand)) = (query.brand(), candidate.brand()) {
                let message = format!("did you mean `{}`?", candidate_brand);

                Err(ResolveError {
                    kind: ErrorKind::UndefinedSymbol(query_brand.clone()),
                    span: query_brand.span.clone(),
                    hints: vec![Hint {
                        message,
                        action: vec![],
                    }],
                    note: None,
                })
            } else {
                Ok(Resemblance::from(resemblance))
            }
        } else {
            Ok(Resemblance::from(resemblance))
        }
    }
}

#[derive(Debug)]
pub struct ScopeProximity;

impl Resembler<Element, Symbol, ResolveError> for ScopeProximity {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let resemblance = match (query.brand(), candidate.brand()) {
            (Some(_), Some(_)) => 0.65,
            _ => 0.0,
        };

        // Only create scope errors for very low similarity
        if resemblance > 0.2 && resemblance < 0.4 {
            if let (Some(query_brand), Some(candidate_brand)) = (query.brand(), candidate.brand()) {
                let message = format!("symbol `{}` is not in scope, did you mean `{}`?", query_brand, candidate_brand);

                Err(ResolveError {
                    kind: ErrorKind::UndefinedSymbol(query_brand.clone()),
                    span: query_brand.span.clone(),
                    hints: vec![Hint {
                        message,
                        action: vec![],
                    }],
                    note: None,
                })
            } else {
                Ok(Resemblance::from(resemblance))
            }
        } else {
            Ok(Resemblance::from(resemblance))
        }
    }
}

#[derive(Debug)]
pub struct PartialIdentifier {
    min_length: usize,
}

impl Default for PartialIdentifier {
    fn default() -> Self {
        Self {
            min_length: 3,
        }
    }
}

impl Resembler<Element, Symbol, ResolveError> for PartialIdentifier {
    fn resemblance(&self, query: &Element, candidate: &Symbol) -> Result<Resemblance, ResolveError> {
        let resemblance = match &query.kind {
            ElementKind::Identifier(query_ident) => {
                if query_ident.len() < self.min_length {
                    return Ok(Resemblance::from(0.0));
                }
                match candidate.brand() {
                    Some(candidate_token) => {
                        let query_lower = query_ident.to_lowercase();
                        let candidate_lower = candidate_token.to_string().to_lowercase();
                        if candidate_lower.contains(&query_lower) || query_lower.contains(&candidate_lower) {
                            0.75 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                        } else {
                            0.0
                        }
                    }
                    None => 0.0,
                }
            }
            _ => 0.0,
        };

        // Only create partial match errors for moderate similarity
        if resemblance > 0.3 && resemblance < 0.6 {
            if let (Some(query_brand), Some(candidate_brand)) = (query.brand(), candidate.brand()) {
                let message = format!("partial match found, did you mean `{}`?", candidate_brand);

                Err(ResolveError {
                    kind: ErrorKind::UndefinedSymbol(query_brand.clone()),
                    span: query_brand.span.clone(),
                    hints: vec![Hint {
                        message,
                        action: vec![],
                    }],
                    note: None,
                })
            } else {
                Ok(Resemblance::from(resemblance))
            }
        } else {
            Ok(Resemblance::from(resemblance))
        }
    }
}

impl PartialEq<Symbol> for Element {
    fn eq(&self, other: &Symbol) -> bool {
        match (&self.kind, &other.kind) {
            (ElementKind::Identifier(ident), SymbolKind::Binding { target, .. }) => {
                if let ElementKind::Identifier(target_ident) = &target.kind {
                    ident == target_ident
                } else {
                    false
                }
            }
            (ElementKind::Constructor { name: expr_name, .. }, SymbolKind::Structure { name: struct_name, .. }) => {
                expr_name.brand() == struct_name.brand()
            }
            (ElementKind::Constructor { name: expr_name, .. }, SymbolKind::Enumeration { name: enum_name, .. }) => {
                expr_name.brand() == enum_name.brand()
            }
            _ => false,
        }
    }
}

pub fn symbol_matcher() -> Assessor<Element, Symbol, ResolveError> {
    Assessor::<Element, Symbol, ResolveError>::new()
        .floor(0.65)
        .dimension(FullMatcher::default(), 0.30)
        .dimension(SymbolType, 0.25)
        .dimension(ParameterCount, 0.20)
        .dimension(ContextualRelevance::default(), 0.20)
        .dimension(ScopeProximity, 0.05)
}