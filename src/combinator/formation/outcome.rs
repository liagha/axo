#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Panicked,
    Aligned,
    Failed,
    Blank,
    Ignored,
    Custom(i8),
}

impl Outcome {
    #[inline]
    pub const fn priority(self) -> i8 {
        match self {
            Outcome::Panicked => 4,
            Outcome::Failed => 3,
            Outcome::Aligned => 2,
            Outcome::Ignored => 1,
            Outcome::Blank => 0,
            Outcome::Custom(v) => v,
        }
    }

    #[inline]
    pub const fn is_productive(self) -> bool {
        matches!(self, Outcome::Aligned | Outcome::Failed)
    }

    #[inline]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Outcome::Panicked | Outcome::Failed)
    }

    #[inline]
    pub const fn is_neutral(self) -> bool {
        matches!(self, Outcome::Blank | Outcome::Ignored)
    }

    #[inline]
    pub const fn is_success(self) -> bool {
        matches!(self, Outcome::Aligned)
    }

    #[inline]
    pub fn escalate(self, other: Outcome) -> Outcome {
        if other.priority() > self.priority() {
            other
        } else {
            self
        }
    }

    #[inline]
    pub fn demote(self) -> Outcome {
        match self {
            Outcome::Panicked => Outcome::Failed,
            Outcome::Aligned => Outcome::Ignored,
            other => other,
        }
    }
}

impl From<Outcome> for i8 {
    fn from(val: Outcome) -> i8 {
        match val {
            Outcome::Panicked => 127,
            Outcome::Aligned => 1,
            Outcome::Failed => 0,
            Outcome::Blank => -1,
            Outcome::Ignored => -2,
            Outcome::Custom(value) => value,
        }
    }
}

impl From<i8> for Outcome {
    fn from(value: i8) -> Outcome {
        match value {
            127 => Outcome::Panicked,
            1 => Outcome::Aligned,
            0 => Outcome::Failed,
            -1 => Outcome::Blank,
            -2 => Outcome::Ignored,
            value => Outcome::Custom(value),
        }
    }
}
