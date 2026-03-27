use crate::{char_property, chars};

char_property! {
    pub struct Control(bool) {
        abbr => "Control";
        long => "Control";
        human => "Control";

        data_table_path => "tables/control.rsv";
    }

    pub fn is_control(char) -> bool;
}
