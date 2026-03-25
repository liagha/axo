use {
    crate::{
        data::Str,
        format::{Result as FormatResult, Formatter, Display}
    }
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
        Self {
            name: String::new(),
            variant: None,
            fields: Vec::new(),
            open: "(".to_string(),
            close: ")".to_string(),
            separator: ", ".to_string(),
            variant_separator: ".".to_string(),
            maximum: 10,
            indent: 4,
            show_head: false,
            show_variant: true,
            show_name: false,
            inline: true,
            block: false,
            trailing: false,
            space: false,
            fold: false,
            text: None,
        }
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

        let mut head = String::new();
        if self.show_head {
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

        let mut items = Vec::new();
        for (key, val) in &self.fields {
            let out = val.build(depth + 1);
            if self.show_name && !key.is_empty() {
                items.push(format!("{}: {}", key, out));
            } else {
                items.push(out);
            }
        }

        let joined = items.join(&self.separator);
        let mut flat = head.clone();

        let show_delimiters = !head.is_empty() || items.len() != 1;

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

        if self.inline || (!self.block && flat.len() <= self.maximum) {
            return flat;
        }

        let pad = " ".repeat(depth * self.indent);
        let inner = " ".repeat((depth + 1) * self.indent);
        let mut tree = head;

        if show_delimiters && !self.open.is_empty() {
            if !tree.is_empty() {
                tree.push(' ');
            }
            tree.push_str(&self.open);
        }
        tree.push('\n');

        for (i, item) in items.iter().enumerate() {
            tree.push_str(&inner);
            tree.push_str(item);
            if i < items.len() - 1 || self.trailing {
                tree.push_str(&self.separator);
            }
            tree.push('\n');
        }

        tree.push_str(&pad);
        if show_delimiters {
            tree.push_str(&self.close);
        }

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
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        write!(f, "{}", self.format())
    }
}

impl<'str> From<Stencil> for Str<'str> {
    fn from(value: Stencil) -> Self {
        Str::from(value.to_string())
    }
}