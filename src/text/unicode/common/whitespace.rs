use crate::{char_property, chars};

char_property! {
    pub struct WhiteSpace(bool) {
        abbr => "WSpace";
        long => "White_Space";
        human => "White Space";

        data_table_path => "tables/whitespace.rsv";
    }

    pub fn is_whitespace(char) -> bool;
}

