use crate::{char_property, chars};

char_property! {
    pub struct Numeric(bool) {
        abbr => "Numeric";
        long => "Numeric";
        human => "Numeric";

        data_table_path => "tables/numeric.rsv";
    }

    pub fn is_numeric(char) -> bool;
}
