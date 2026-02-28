use crate::{char_property, chars};

char_property! {
    pub struct Alphabetic(bool) {
        abbr => "Alpha";
        long => "Alphabetic";
        human => "Alphabetic";

        data_table_path => "tables/alphabetic.rsv";
    }

    pub fn is_alphabetic(char) -> bool;
}

