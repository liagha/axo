use {
    crate::{
        internal::Hash,
        format::Debug
    }
};

pub trait CharProperty: PartialCharProperty + Debug + Eq + Hash {
    fn prop_abbr_name() -> &'static str;

    fn prop_long_name() -> &'static str;

    fn prop_human_name() -> &'static str;
}

pub trait PartialCharProperty: Copy {
    fn of(ch: char) -> Option<Self>;
}

pub trait TotalCharProperty: PartialCharProperty + Default {
    fn of(ch: char) -> Self;
}

impl<T: TotalCharProperty> PartialCharProperty for T {
    fn of(ch: char) -> Option<Self> {
        Some(<Self as TotalCharProperty>::of(ch))
    }
}
