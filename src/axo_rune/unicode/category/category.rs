#![allow(dead_code)]

use crate::axo_rune::unicode::TotalCharProperty;
use crate::char_property;

char_property! {
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
    use crate::axo_rune::unicode::tables::CharDataTable;
    use crate::chars;
    use crate::unicode::category::category::abbr_names::*;

    pub const GENERAL_CATEGORY_TABLE: CharDataTable<super::GeneralCategory> =
        include!("tables/general_category.rsv");
}

impl GeneralCategory {
    /// Find the `GeneralCategory` of a single char.
    pub fn of(ch: char) -> GeneralCategory {
        data::GENERAL_CATEGORY_TABLE.find_or_default(ch)
    }
}

impl GeneralCategory {
    /// `Lu` | `Ll` | `Lt`  (Short form: `LC`)
    pub fn is_cased_letter(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Lu | Ll | Lt)
    }

    /// `Lu` | `Ll` | `Lt` | `Lm` | `Lo`  (Short form: `L`)
    pub fn is_letter(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Lu | Ll | Lt | Lm | Lo)
    }

    /// `Mn` | `Mc` | `Me`  (Short form: `M`)
    pub fn is_mark(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Mn | Mc | Me)
    }

    /// `Nd` | `Nl` | `No`  (Short form: `N`)
    pub fn is_number(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Nd | Nl | No)
    }

    /// `Pc` | `Pd` | `Ps` | `Pe` | `Pi` | `Pf` | `Po`  (Short form: `P`)
    pub fn is_punctuation(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Pc | Pd | Ps | Pe | Pi | Pf | Po)
    }

    /// `Sm` | `Sc` | `Sk` | `So`  (Short form: `S`)
    pub fn is_symbol(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Sm | Sc | Sk | So)
    }

    /// `Zs` | `Zl` | `Zp`  (Short form: `Z`)
    pub fn is_separator(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Zs | Zl | Zp)
    }

    /// `Cc` | `Cf` | `Cs` | `Co` | `Cn`  (Short form: `C`)
    pub fn is_other(&self) -> bool {
        use self::abbr_names::*;
        matches!(*self, Cc | Cf | Cs | Co | Cn)
    }
}

#[cfg(test)]
mod tests {
    use super::GeneralCategory as GC;
    use core::char;
    use crate::EnumeratedCharProperty;

    #[test]
    fn test_ascii() {
        for c in 0x00..(0x1F + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::Control);
        }

        assert_eq!(GC::of(' '), GC::SpaceSeparator);
        assert_eq!(GC::of('!'), GC::OtherPunctuation);
        assert_eq!(GC::of('"'), GC::OtherPunctuation);
        assert_eq!(GC::of('#'), GC::OtherPunctuation);
        assert_eq!(GC::of('$'), GC::CurrencySymbol);
        assert_eq!(GC::of('%'), GC::OtherPunctuation);
        assert_eq!(GC::of('&'), GC::OtherPunctuation);
        assert_eq!(GC::of('\''), GC::OtherPunctuation);
        assert_eq!(GC::of('('), GC::OpenPunctuation);
        assert_eq!(GC::of(')'), GC::ClosePunctuation);
        assert_eq!(GC::of('*'), GC::OtherPunctuation);
        assert_eq!(GC::of('+'), GC::MathSymbol);
        assert_eq!(GC::of(','), GC::OtherPunctuation);
        assert_eq!(GC::of('-'), GC::DashPunctuation);
        assert_eq!(GC::of('.'), GC::OtherPunctuation);
        assert_eq!(GC::of('/'), GC::OtherPunctuation);

        for c in ('0' as u32)..('9' as u32 + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::DecimalNumber);
        }

        assert_eq!(GC::of(':'), GC::OtherPunctuation);
        assert_eq!(GC::of(';'), GC::OtherPunctuation);
        assert_eq!(GC::of('<'), GC::MathSymbol);
        assert_eq!(GC::of('='), GC::MathSymbol);
        assert_eq!(GC::of('>'), GC::MathSymbol);
        assert_eq!(GC::of('?'), GC::OtherPunctuation);
        assert_eq!(GC::of('@'), GC::OtherPunctuation);

        for c in ('A' as u32)..('Z' as u32 + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::UppercaseLetter);
        }

        assert_eq!(GC::of('['), GC::OpenPunctuation);
        assert_eq!(GC::of('\\'), GC::OtherPunctuation);
        assert_eq!(GC::of(']'), GC::ClosePunctuation);
        assert_eq!(GC::of('^'), GC::ModifierSymbol);
        assert_eq!(GC::of('_'), GC::ConnectorPunctuation);
        assert_eq!(GC::of('`'), GC::ModifierSymbol);

        for c in ('a' as u32)..('z' as u32 + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::LowercaseLetter);
        }

        assert_eq!(GC::of('{'), GC::OpenPunctuation);
        assert_eq!(GC::of('|'), GC::MathSymbol);
        assert_eq!(GC::of('}'), GC::ClosePunctuation);
        assert_eq!(GC::of('~'), GC::MathSymbol);
    }

    #[test]
    fn test_bmp_edge() {
        // 0xFEFF ZERO WIDTH NO-BREAK SPACE (or) BYTE ORDER MARK
        let bom = '\u{FEFF}';
        assert_eq!(GC::of(bom), GC::Format);
        // 0xFFFC OBJECT REPLACEMENT CHARACTER
        assert_eq!(GC::of('￼'), GC::OtherSymbol);
        // 0xFFFD REPLACEMENT CHARACTER
        assert_eq!(GC::of('�'), GC::OtherSymbol);

        for &c in [0xFFEF, 0xFFFE, 0xFFFF].iter() {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::Unassigned);
        }
    }

    #[test]
    fn test_private_use() {
        for c in 0xF_0000..(0xF_FFFD + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::PrivateUse);
        }

        for c in 0x10_0000..(0x10_FFFD + 1) {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::PrivateUse);
        }

        for &c in [0xF_FFFE, 0xF_FFFF, 0x10_FFFE, 0x10_FFFF].iter() {
            let c = char::from_u32(c).unwrap();
            assert_eq!(GC::of(c), GC::Unassigned);
        }
    }

    #[test]
    fn test_abbr_name() {
        assert_eq!(GC::UppercaseLetter.abbr_name(), "Lu");
        assert_eq!(GC::Unassigned.abbr_name(), "Cn");
    }

    #[test]
    fn test_long_name() {
        assert_eq!(GC::UppercaseLetter.long_name(), "Uppercase_Letter");
        assert_eq!(GC::Unassigned.long_name(), "Unassigned");
    }

    #[test]
    fn test_human_name() {
        assert_eq!(GC::UppercaseLetter.human_name(), "Uppercase Letter");
        assert_eq!(GC::Unassigned.human_name(), "Unassigned");
    }
}
