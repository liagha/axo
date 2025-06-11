#[macro_export]
macro_rules! char_property {

    (
        $(#[$prop_meta:meta])*
        pub enum $prop_name:ident {
            abbr => $prop_abbr:expr;
            long => $prop_long:expr;
            human => $prop_human:expr;

            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident {
                    abbr => $variant_abbr:ident,
                    long => $variant_long:ident,
                    human => $variant_human:expr,
                }
            )*
        }

        $(#[$abbr_mod_meta:meta])*
        pub mod $abbr_mod:ident for abbr;

        $(#[$long_mod_meta:meta])*
        pub mod $long_mod:ident for long;

    ) => {
        $(#[$prop_meta])*
        #[allow(bad_style)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        pub enum $prop_name {
            $( $(#[$variant_meta])* $variant_name, )*
        }

        $(#[$abbr_mod_meta])*
        #[allow(bad_style)]
        pub mod $abbr_mod {
            $( pub use super::$prop_name::$variant_name as $variant_abbr; )*
        }

        $(#[$long_mod_meta])*

        char_property! {
            __impl FromStr for $prop_name;
            $(
                stringify!($variant_abbr) => $prop_name::$variant_name;
                stringify!($variant_long) => $prop_name::$variant_name;
            )*
        }

        char_property! {
            __impl CharProperty for $prop_name;
            $prop_abbr;
            $prop_long;
            $prop_human;
        }

        char_property! {
            __impl Display for $prop_name by EnumeratedCharProperty
        }

        impl $crate::EnumeratedCharProperty for $prop_name {
            fn all_values() -> &'static [$prop_name] {
                const VALUES: &[$prop_name] = &[
                    $( $prop_name::$variant_name, )*
                ];
                VALUES
            }
            fn abbr_name(&self) -> &'static str {
                match *self {
                    $( $prop_name::$variant_name => stringify!($variant_abbr), )*
                }
            }
            fn long_name(&self) -> &'static str {
                match *self {
                    $( $prop_name::$variant_name => stringify!($variant_long), )*
                }
            }
            fn human_name(&self) -> &'static str {
                match *self {
                    $( $prop_name::$variant_name => $variant_human, )*
                }
            }
        }
    };

    (
        $(#[$prop_meta:meta])*
        pub struct $prop_name:ident(bool) {
            abbr => $prop_abbr:expr;
            long => $prop_long:expr;
            human => $prop_human:expr;

            data_table_path => $data_path:expr;
        }

        $(#[$is_fn_meta:meta])*
        pub fn $is_fn:ident(char) -> bool;

    ) => {
        $(#[$prop_meta])*
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
        pub struct $prop_name(bool);

        $(#[$is_fn_meta])*
        pub fn $is_fn(ch: char) -> bool {
            $prop_name::of(ch).as_bool()
        }

        impl $prop_name {
            pub fn of(ch: char) -> Self {
                use $crate::axo_text::tables::CharDataTable;
                const TABLE: CharDataTable<()> = include!($data_path);
                $prop_name(TABLE.contains(ch))
            }

            pub fn as_bool(&self) -> bool { self.0 }
        }

        char_property! {
            __impl FromStr for $prop_name;
            "y" => $prop_name(true);
            "yes" => $prop_name(true);
            "t" => $prop_name(true);
            "true" => $prop_name(true);
            
            "n" => $prop_name(false);
            "no" => $prop_name(false);
            "f" => $prop_name(false);
            "false" => $prop_name(false);
        }

        char_property! {
            __impl CharProperty for $prop_name;
            $prop_abbr;
            $prop_long;
            $prop_human;
        }

        impl $crate::TotalCharProperty for $prop_name {
            fn of(ch: char) -> Self { Self::of(ch) }
        }

        impl $crate::BinaryCharProperty for $prop_name {
            fn as_bool(&self) -> bool { self.as_bool() }
        }

        impl From<$prop_name> for bool {
            fn from(prop: $prop_name) -> bool { prop.as_bool() }
        }

        char_property! {
            __impl Display for $prop_name by BinaryCharProperty
        }
    };

    (
        __impl CharProperty for $prop_name:ident;
        $prop_abbr:expr;
        $prop_long:expr;
        $prop_human:expr;
    ) => {
        impl $crate::CharProperty for $prop_name {
            fn prop_abbr_name() -> &'static str { $prop_abbr }
            fn prop_long_name() -> &'static str { $prop_long }
            fn prop_human_name() -> &'static str { $prop_human }
        }
    };

    (
        __impl FromStr for $prop_name:ident;
        $( $id:expr => $value:expr; )*
    ) => {
        #[allow(unreachable_patterns)]
        impl $crate::__str::FromStr for $prop_name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( $id => Ok($value), )*
                    $( s if s.eq_ignore_ascii_case($id) => Ok($value), )*
                    _ => Err(()),
                }
            }
        }
    };

    ( __impl Display for $prop_name:ident by $trait:ident ) => {
        impl $crate::axo_text::__fmt::Display for $prop_name {
            fn fmt(&self, f: &mut $crate::__fmt::Formatter) -> $crate::__fmt::Result {
                $crate::$trait::human_name(self).fmt(f)
            }
        }
    };
}
