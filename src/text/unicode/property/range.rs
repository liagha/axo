use super::property::CharProperty;

pub trait EnumeratedCharProperty: Sized + CharProperty {
    fn all_values() -> &'static [Self];

    fn abbr_name(&self) -> &'static str;

    fn long_name(&self) -> &'static str;

    fn human_name(&self) -> &'static str;
}

pub trait BinaryCharProperty: CharProperty {
    fn as_bool(&self) -> bool;

    fn abbr_name(&self) -> &'static str {
        if self.as_bool() {
            "Y"
        } else {
            "N"
        }
    }

    fn long_name(&self) -> &'static str {
        if self.as_bool() {
            "Yes"
        } else {
            "No"
        }
    }

    fn human_name(&self) -> &'static str {
        if self.as_bool() {
            "Yes"
        } else {
            "No"
        }
    }
}

pub trait NumericCharPropertyValue {}

impl NumericCharPropertyValue for u8 {}

pub trait NumericCharProperty<NumericValue: NumericCharPropertyValue>: CharProperty {
    fn number(&self) -> NumericValue;
}

pub trait CustomCharProperty<Value>: CharProperty {
    fn actual(&self) -> Value;
}
