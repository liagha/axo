use crate::{char_property, text::unicode::TotalCharProperty};

char_property! {
    use crate::axo_text::EnumeratedCharProperty;

    pub enum GeneralCategory {
        abbr => "gc";
        long => "General_Category";
        human => "General Category";

        UppercaseLetter {
            abbr => Lu,
            long => Uppercase_Letter,
            human => "Uppercase Letter",
        }

        LowercaseLetter {
            abbr => Ll,
            long => Lowercase_Letter,
            human => "Lowercase Letter",
        }

        TitlecaseLetter {
            abbr => Lt,
            long => Titlecase_Letter,
            human => "Titlecase Letter",
        }

        ModifierLetter {
            abbr => Lm,
            long => Modifier_Letter,
            human => "Modifier Letter",
        }

        OtherLetter {
            abbr => Lo,
            long => Other_Letter,
            human => "Other Letter",
        }

        NonspacingMark {
            abbr => Mn,
            long => Nonspacing_Mark,
            human => "Nonspacing Mark",
        }

        SpacingMark {
            abbr => Mc,
            long => Spacing_Mark,
            human => "Spacing Mark",
        }

        EnclosingMark {
            abbr => Me,
            long => Enclosing_Mark,
            human => "Enclosing Mark",
        }

        DecimalNumber {
            abbr => Nd,
            long => Decimal_Number,
            human => "Decimal Digit",
        }

        LetterNumber {
            abbr => Nl,
            long => Letter_Number,
            human => "Letterlike Number",
        }

        OtherNumber {
            abbr => No,
            long => Other_Number,
            human => "Other Numeric",
        }

        ConnectorPunctuation {
            abbr => Pc,
            long => Connector_Punctuation,
            human => "Connecting Punctuation",
        }

        DashPunctuation {
            abbr => Pd,
            long => Dash_Punctuation,
            human => "Dash Punctuation",
        }

        OpenPunctuation {
            abbr => Ps,
            long => Open_Punctuation,
            human => "Opening Punctuation",
        }

        ClosePunctuation {
            abbr => Pe,
            long => Close_Punctuation,
            human => "Closing Punctuation",
        }

        InitialPunctuation {
            abbr => Pi,
            long => Initial_Punctuation,
            human => "Initial Quotation",
        }

        FinalPunctuation {
            abbr => Pf,
            long => Final_Punctuation,
            human => "Final Quotation",
        }

        OtherPunctuation {
            abbr => Po,
            long => Other_Punctuation,
            human => "Other Punctuation",
        }

        MathSymbol {
            abbr => Sm,
            long => Math_Symbol,
            human => "Math Symbol",
        }

        CurrencySymbol {
            abbr => Sc,
            long => Currency_Symbol,
            human => "Currency Symbol",
        }

        ModifierSymbol {
            abbr => Sk,
            long => Modifier_Symbol,
            human => "Modifier Symbol",
        }

        OtherSymbol {
            abbr => So,
            long => Other_Symbol,
            human => "Other Symbol",
        }

        SpaceSeparator {
            abbr => Zs,
            long => Space_Separator,
            human => "Space",
        }

        LineSeparator {
            abbr => Zl,
            long => Line_Separator,
            human => "Line Separator",
        }

        ParagraphSeparator {
            abbr => Zp,
            long => Paragraph_Separator,
            human => "Paragraph Separator",
        }

        Control {
            abbr => Cc,
            long => Control,
            human => "Control",
        }

        Format {
            abbr => Cf,
            long => Format,
            human => "Formatting",
        }

        Surrogate {
            abbr => Cs,
            long => Surrogate,
            human => "Surrogate",
        }

        PrivateUse {
            abbr => Co,
            long => Private_Use,
            human => "Private-Use",
        }

        Unassigned {
            abbr => Cn,
            long => Unassigned,
            human => "Unassigned",
        }
    }

    pub mod abbr_names for abbr;
    pub mod long_names for long;
}

impl TotalCharProperty for GeneralCategory {
    fn of(ch: char) -> Self {
        Self::of(ch)
    }
}

impl Default for GeneralCategory {
    fn default() -> Self {
        GeneralCategory::Unassigned
    }
}

mod data {
    use crate::{
        chars,
        text::unicode::{category::category::abbr_names::*, tables::CharDataTable},
    };

    pub const GENERAL_CATEGORY_TABLE: CharDataTable<super::GeneralCategory> =
        include!("tables/general.rsv");
}

impl GeneralCategory {
    pub fn of(ch: char) -> GeneralCategory {
        data::GENERAL_CATEGORY_TABLE.find_or_default(ch)
    }
}

impl GeneralCategory {
    pub fn is_cased_letter(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Lu | Ll | Lt)
    }

    pub fn is_letter(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Lu | Ll | Lt | Lm | Lo)
    }

    pub fn is_mark(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Mn | Mc | Me)
    }

    pub fn is_number(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Nd | Nl | No)
    }

    pub fn is_punctuation(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Pc | Pd | Ps | Pe | Pi | Pf | Po)
    }

    pub fn is_symbol(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Sm | Sc | Sk | So)
    }

    pub fn is_separator(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Zs | Zl | Zp)
    }

    pub fn is_other(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Cc | Cf | Cs | Co | Cn)
    }
}

