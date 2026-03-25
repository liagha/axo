use crate::{data::Str, format::Stencil, internal::hash::Set};

pub trait Show<'show> {
    fn format(&self, config: Stencil) -> Stencil;
    fn indent(&self, stencil: Stencil) -> Str<'show> {
        Str::from(
            self.format(stencil)
                .to_string()
                .lines()
                .into_iter()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl<'show, T: Show<'show>> Show<'show> for &T {
    fn format(&self, config: Stencil) -> Stencil {
        (*self).format(config)
    }
}

impl<'show, T: Show<'show>> Show<'show> for Box<T> {
    fn format(&self, config: Stencil) -> Stencil {
        (**self).format(config)
    }
}

impl<'show, T: Show<'show>> Show<'show> for Option<T> {
    fn format(&self, config: Stencil) -> Stencil {
        match self {
            Some(value) => config.clone().new("Some").field("", value.format(config)),
            None => Stencil::from("None"),
        }
    }
}

impl<'show, Item: Show<'show>> Show<'show> for [Item] {
    fn format(&self, config: Stencil) -> Stencil {
        if self.is_empty() {
            return Stencil::from("[]");
        }
        let mut stencil = config.clone();
        for item in self {
            stencil = stencil.field("", item.format(config.clone()));
        }
        stencil
    }
}

impl<'show, Item: Show<'show>> Show<'show> for Vec<Item> {
    fn format(&self, config: Stencil) -> Stencil {
        self.as_slice().format(config)
    }
}

impl<'show, Item: Show<'show>> Show<'show> for Set<Item> {
    fn format(&self, config: Stencil) -> Stencil {
        self.iter().collect::<Vec<&Item>>().format(config)
    }
}

impl<'show> Show<'show> for String {
    fn format(&self, _config: Stencil) -> Stencil {
        Stencil::from(self.clone())
    }
}

impl<'show> Show<'show> for &str {
    fn format(&self, _config: Stencil) -> Stencil {
        Stencil::from(*self)
    }
}

impl<'show> Show<'show> for Str<'show> {
    fn format(&self, _config: Stencil) -> Stencil {
        Stencil::from(self.to_string())
    }
}
