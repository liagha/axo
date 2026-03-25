use crate::{
    format::{Show, Stencil},
    resolver::{Type, TypeKind},
};

impl<'typing> Show<'typing> for Type<'typing> {
    fn format(&self, config: Stencil) -> Stencil {
        config.clone().new("Type").field("kind", self.kind.format(config.clone()))
    }
}

impl<'typing> Show<'typing> for TypeKind<'typing> {
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("TypeKind");
        match self {
            TypeKind::Integer { size, signed } => base.variant("Integer")
                .field("size", size.to_string())
                .field("signed", signed.to_string()),
            TypeKind::Float { size } => base.variant("Float")
                .field("size", size.to_string()),
            TypeKind::Boolean => base.variant("Boolean"),
            TypeKind::String => base.variant("String"),
            TypeKind::Character => base.variant("Character"),
            TypeKind::Pointer { target } => base.variant("Pointer").field("target", target.format(config.clone())),
            TypeKind::Array { member, size } => base.variant("Array")
                .field("member", member.format(config.clone()))
                .field("size", size.to_string()),
            TypeKind::Tuple { members } => base.variant("Tuple").field("members", members.format(config.clone())),
            TypeKind::Function(name, members, output) => base.variant("Function")
                .field("name", name.format(config.clone()))
                .field("members", members.format(config.clone()))
                .field("output", output.format(config.clone())),
            TypeKind::Variable(variable) => base.variant("Variable").field("id", variable.to_string()),
            TypeKind::Void => base.variant("Void"),
            TypeKind::Unknown => base.variant("Unknown"),
            TypeKind::Structure(structure) => base.variant("Structure").field("structure", structure.format(config.clone())),
            TypeKind::Union(union) => base.variant("Union").field("union", union.format(config.clone())),
        }
    }
}
