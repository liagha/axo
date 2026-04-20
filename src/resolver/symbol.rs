use crate::{
    data::{memory::replace, Aggregate, Binding as TypeBinding, Function, Interface},
    parser::{Symbol, SymbolKind},
    resolver::{scope::Scope, Resolvable, Resolver, Type, TypeKind},
};

impl<'a> Symbol<'a> {
    fn value(typing: &Type<'a>) -> Type<'a> {
        match &typing.kind {
            TypeKind::Binding(binding) => binding
                .value
                .as_deref()
                .cloned()
                .or_else(|| binding.annotation.as_deref().cloned())
                .unwrap_or_else(|| Type::from(TypeKind::Unknown)),
            _ => typing.clone(),
        }
    }

    fn seed(identity: crate::data::Identity, kind: &SymbolKind<'a>) -> Option<Type<'a>> {
        match kind {
            SymbolKind::Structure(structure) => Some(Type::new(
                identity,
                TypeKind::Structure(Box::new(Aggregate::new(
                    structure.target.target().unwrap().into(),
                    Vec::new(),
                ))),
            )),
            SymbolKind::Union(union) => Some(Type::new(
                identity,
                TypeKind::Union(Box::new(Aggregate::new(
                    union.target.target().unwrap().into(),
                    Vec::new(),
                ))),
            )),
            SymbolKind::Module(module) => Some(Type::new(
                identity,
                TypeKind::Module(module.target.target().unwrap().into()),
            )),
            _ => None,
        }
    }

    fn bind(
        identity: crate::data::Identity,
        target: crate::data::Str<'a>,
        value: Type<'a>,
        annotation: Option<Type<'a>>,
        kind: crate::data::BindingKind,
    ) -> Type<'a> {
        Type::new(
            identity,
            TypeKind::Binding(Box::new(TypeBinding::new(
                target,
                Some(Box::new(value)),
                annotation.map(Box::new),
                kind,
            ))),
        )
    }

    fn shape(
        identity: crate::data::Identity,
        target: crate::data::Str<'a>,
        members: Vec<Type<'a>>,
        union: bool,
    ) -> Type<'a> {
        let kind = if union {
            TypeKind::Union(Box::new(Aggregate::new(target.into(), members)))
        } else {
            TypeKind::Structure(Box::new(Aggregate::new(target.into(), members)))
        };

        Type::new(identity, kind)
    }

    fn declare_scope(
        resolver: &mut Resolver<'a>,
        members: &mut Vec<crate::parser::Symbol<'a>>,
    ) -> Scope {
        let (_, scope) = resolver.nest(|resolver| {
            for member in members {
                member.declare(resolver);
            }
        });

        scope
    }

    fn resolve_scope(
        resolver: &mut Resolver<'a>,
        scope: Scope,
        members: &mut Vec<crate::parser::Symbol<'a>>,
    ) -> (Vec<Type<'a>>, Scope) {
        let (layout, scope) = resolver.within(scope, |resolver| {
            let mut layout = Vec::new();
            for member in members {
                member.resolve(resolver);
                if member.is_instance() {
                    layout.push(member.typing.clone());
                }
            }
            layout
        });

        (layout, scope)
    }
}

impl<'a> Resolvable<'a> for Symbol<'a> {
    fn declare(&mut self, resolver: &mut Resolver<'a>) {
        if let Some(typing) = Self::seed(self.identity, &self.kind) {
            self.typing = typing;
            resolver.insert(self.clone());
        }

        self.typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                binding.target.declare(resolver);

                let annotation = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);
                    resolver.annotation(annotation).unwrap_or_else(|_| resolver.fresh())
                });

                let value = annotation.clone().unwrap_or_else(|| resolver.fresh());
                binding.target.typing = value.clone();

                Self::bind(
                    self.identity,
                    binding.target.target().unwrap_or_default(),
                    value,
                    annotation,
                    binding.kind,
                )
            }
            SymbolKind::Function(function) => {
                let target = function.target.target().unwrap();
                let (typing, scope) = resolver.nest(|resolver| {
                    for member in &mut function.members {
                        member.declare(resolver);
                    }

                    let members = function
                        .members
                        .iter()
                        .map(|member| Self::value(&member.typing))
                        .collect();

                    let mut body = false;
                    let output = match &mut function.output {
                        Some(annotation) => {
                            annotation.resolve(resolver);
                            match resolver.annotation(annotation) {
                                Ok(typing) => typing,
                                Err(_) => {
                                    body = true;
                                    resolver.fresh()
                                }
                            }
                        }
                        None => resolver.fresh(),
                    };

                    if body && function.body.is_none() {
                        function.body = function.output.take();
                    }

                    Type::new(
                        self.identity,
                        TypeKind::Function(Box::new(Function::new(
                            target.into(),
                            members,
                            resolver.fresh(),
                            Some(Box::new(output)),
                            Interface::Axo,
                            false,
                            false,
                        ))),
                    )
                });

                self.scope = Box::new(scope);
                self.scope.parent = None;
                typing
            }
            SymbolKind::Structure(structure) => {
                self.scope = Box::new(Self::declare_scope(resolver, &mut structure.members));
                self.scope.parent = None;
                self.typing.clone()
            }
            SymbolKind::Union(union) => {
                self.scope = Box::new(Self::declare_scope(resolver, &mut union.members));
                self.scope.parent = None;
                self.typing.clone()
            }
            SymbolKind::Module(_) => self.typing.clone(),
        };

        resolver.insert(self.clone());
    }

    fn resolve(&mut self, resolver: &mut Resolver<'a>) {
        let expected = self.typing.clone();

        let typing = match &mut self.kind {
            SymbolKind::Binding(binding) => {
                let annotation = binding.annotation.as_mut().map(|annotation| {
                    annotation.resolve(resolver);
                    match resolver.annotation(annotation) {
                        Ok(typing) => typing,
                        Err(error) => {
                            resolver.errors.push(error);
                            resolver.fresh()
                        }
                    }
                });

                let value = binding.value.as_mut().map(|value| {
                    value.resolve(resolver);
                    value.typing.clone()
                });

                let value = match (annotation.clone(), value) {
                    (Some(left), Some(right)) => resolver.unify(self.span, &left, &right),
                    (Some(left), None) => left,
                    (None, Some(right)) => right,
                    (None, None) => resolver.fresh(),
                };

                resolver.unify(self.span, &binding.target.typing, &value);
                binding.target.typing = value.clone();

                Self::bind(
                    self.identity,
                    binding.target.target().unwrap_or_default(),
                    value,
                    annotation,
                    binding.kind,
                )
            }
            SymbolKind::Structure(structure) => {
                let target = structure.target.target().unwrap();
                let scope = replace(&mut self.scope, Box::new(Scope::new(None)));
                let (members, scope) = Self::resolve_scope(resolver, *scope, &mut structure.members);
                self.scope = Box::new(scope);
                self.scope.parent = None;
                Self::shape(self.identity, target, members, false)
            }
            SymbolKind::Union(union) => {
                let target = union.target.target().unwrap();
                let scope = replace(&mut self.scope, Box::new(Scope::new(None)));
                let (members, scope) = Self::resolve_scope(resolver, *scope, &mut union.members);
                self.scope = Box::new(scope);
                self.scope.parent = None;
                Self::shape(self.identity, target, members, true)
            }
            SymbolKind::Function(function) => {
                let target = function.target.target().unwrap();
                let scope = replace(&mut self.scope, Box::new(Scope::new(None)));
                let (typing, scope) = resolver.within(*scope, |resolver| {
                    let members = function
                        .members
                        .iter_mut()
                        .map(|member| {
                            member.resolve(resolver);
                            Self::value(&member.typing)
                        })
                        .collect::<Vec<_>>();

                    let output = function.output.as_mut().map(|output| {
                        output.resolve(resolver);
                        match resolver.annotation(output) {
                            Ok(typing) => {
                                output.typing = typing.clone();
                                typing
                            }
                            Err(error) => {
                                resolver.errors.push(error);
                                resolver.fresh()
                            }
                        }
                    });

                    let expect = output.clone().unwrap_or_else(|| resolver.fresh());
                    resolver.returns.push(expect.clone());

                    let body = match &mut function.body {
                        Some(body) => {
                            body.resolve(resolver);
                            resolver.unify(self.span, &expect, &body.typing);
                            body.typing.clone()
                        }
                        None => Type::from(TypeKind::Void),
                    };

                    resolver.returns.pop();

                    Type::new(
                        self.identity,
                        TypeKind::Function(Box::new(Function::new(
                            target.into(),
                            members,
                            body,
                            Some(Box::new(resolver.reify(&expect))),
                            Interface::Axo,
                            false,
                            false,
                        ))),
                    )
                });

                self.scope = Box::new(scope);
                self.scope.parent = None;
                typing
            }
            SymbolKind::Module(module) => {
                Type::new(self.identity, TypeKind::Module(module.target.target().unwrap().into()))
            }
        };

        self.typing = resolver.unify(self.span, &expected, &typing);
        resolver.insert(self.clone());
    }

    fn is_instance(&self) -> bool {
        matches!(self.kind, SymbolKind::Binding(_))
    }
}
