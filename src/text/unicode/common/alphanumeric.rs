use crate::{char_property, chars};

char_property! {
    pub struct Alphanumeric(bool) {
        abbr => "Alphanumeric";
        long => "Alphanumeric";
        human => "Alphanumeric";

        data_table_path => "tables/alphanumeric.rsv";
    }

    pub fn is_alphanumeric(char) -> bool;
}
