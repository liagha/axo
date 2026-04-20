use crate::{
    data::Str,
    format::{Display, Formatter, Result},
};

#[derive(Clone)]
pub struct Stencil {
    pub name: String,
    pub variant: Option<String>,
    pub fields: Vec<(String, Stencil)>,
    pub open: String,
    pub close: String,
    pub separator: String,
    pub variant_separator: String,
    pub maximum: usize,
    pub indent: usize,
    pub show_head: bool,
    pub show_variant: bool,
    pub show_name: bool,
    pub inline: bool,
    pub block: bool,
    pub trailing: bool,
    pub space: bool,
    pub fold: bool,
    pub text: Option<String>,
}

impl Default for Stencil {
    fn default() -> Self {
        Self::simple()
    }
}

impl Stencil {
    pub fn new(&self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variant: self.variant.clone(),
            fields: self.fields.clone(),
            open: self.open.clone(),
            close: self.close.clone(),
            separator: self.separator.clone(),
            variant_separator: self.variant_separator.clone(),
            maximum: self.maximum,
            indent: self.indent,
            show_head: self.show_head,
            show_variant: self.show_variant,
            show_name: self.show_name,
            inline: self.inline,
            block: self.block,
            trailing: self.trailing,
            space: self.space,
            fold: self.fold,
            text: None,
        }
    }

    pub fn debug() -> Self {
        Self {
            name: String::new(),
            variant: None,
            fields: Vec::new(),
            open: "(".to_string(),
            close: ")".to_string(),
            separator: ", ".to_string(),
            variant_separator: ".".to_string(),
            maximum: 200,
            indent: 4,
            show_head: true,
            show_variant: true,
            show_name: true,
            inline: false,
            block: false,
            trailing: false,
            space: false,
            fold: false,
            text: None,
        }

    }

    pub fn simple() -> Self {
        Self {
            show_head: false,
            show_name: false,
            fold: true,
            ..Self::debug()
        }
    }

    pub fn variant(mut self, text: impl Into<String>) -> Self {
        self.variant = Some(text.into());
        self
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<Stencil>) -> Self {
        self.fields.push((name.into(), value.into()));
        self
    }

    pub fn inline(mut self) -> Self {
        self.inline = true;
        self.block = false;
        self
    }

    pub fn block(mut self) -> Self {
        self.block = true;
        self.inline = false;
        self
    }

    pub fn format(&self) -> String {
        self.build(0)
    }

    fn build(&self, depth: usize) -> String {
        if let Some(text) = &self.text {
            return text.clone();
        }

        if self.fold && self.fields.len() == 1 && self.variant.is_none() {
            return self.fields[0].1.build(depth);
        }

        let mut active_fields = self.fields.clone();

        if self.fold && active_fields.len() == 1 {
            let inner = &active_fields[0].1;
            let inner_shows_head = inner.show_head && !inner.name.is_empty();

            if inner.variant.is_none() && inner.text.is_none() && !inner_shows_head {
                active_fields = inner.fields.clone();
            }
        }

        let mut head = String::new();
        if self.show_head && !self.name.is_empty() {
            head.push_str(&self.name);
        }

        if self.show_variant {
            if let Some(variant) = &self.variant {
                if !head.is_empty() {
                    head.push_str(&self.variant_separator);
                }
                head.push_str(variant);
            }
        }

        let show_delimiters = !head.is_empty() || active_fields.len() != 1 || self.name.is_empty();
        let child_depth = if show_delimiters { depth + 1 } else { depth };

        let mut items = Vec::new();

        for (key, val) in &active_fields {
            let out = val.build(child_depth);
            if self.show_name && !key.is_empty() {
                items.push(format!("{}: {}", key, out));
            } else {
                items.push(out);
            }
        }

        let joined = items.join(&self.separator);
        let mut flat = head.clone();

        if show_delimiters {
            flat.push_str(&self.open);
        }
        if self.space && !items.is_empty() {
            flat.push(' ');
        }
        flat.push_str(&joined);
        if self.trailing && !items.is_empty() {
            flat.push_str(&self.separator);
        }
        if self.space && !items.is_empty() {
            flat.push(' ');
        }
        if show_delimiters {
            flat.push_str(&self.close);
        }

        if self.inline || (!self.block && !joined.contains('\n') && flat.len() <= self.maximum) {
            return flat;
        }

        if !show_delimiters {
            return items[0].clone();
        }

        let pad = " ".repeat(depth * self.indent);
        let inner_pad = " ".repeat(child_depth * self.indent);
        let mut tree = head;

        if !self.open.is_empty() {
            if !tree.is_empty() {
                tree.push(' ');
            }
            tree.push_str(&self.open);
        }
        tree.push('\n');

        let mut current_line = inner_pad.clone();

        for (i, item) in items.iter().enumerate() {
            let mut chunk = String::new();
            chunk.push_str(item);
            if i < items.len() - 1 || self.trailing {
                chunk.push_str(&self.separator);
            }

            let is_multiline = item.contains('\n');
            let exceeds_max = current_line.len() + chunk.len() > self.maximum;

            if (is_multiline || exceeds_max) && current_line.len() > inner_pad.len() {
                tree.push_str(&current_line);
                tree.push('\n');
                current_line = inner_pad.clone();
            }

            current_line.push_str(&chunk);

            if is_multiline {
                tree.push_str(&current_line);
                tree.push('\n');
                current_line = inner_pad.clone();
            }
        }

        if current_line.len() > inner_pad.len() {
            tree.push_str(&current_line);
            tree.push('\n');
        }

        tree.push_str(&pad);
        tree.push_str(&self.close);

        tree
    }
}

impl From<&str> for Stencil {
    fn from(val: &str) -> Self {
        let mut base = Stencil::default();
        base.text = Some(val.to_string());
        base
    }
}

impl From<String> for Stencil {
    fn from(val: String) -> Self {
        let mut base = Stencil::default();
        base.text = Some(val);
        base
    }
}

impl Display for Stencil {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format())
    }
}

impl<'str> From<Stencil> for Str<'str> {
    fn from(value: Stencil) -> Self {
        Str::from(value.to_string())
    }
}
