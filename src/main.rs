#![allow(dead_code)]
extern crate core;

mod axo_checker;
mod axo_cursor;
mod axo_data;
mod axo_error;
mod axo_form;
mod axo_format;
mod axo_initial;
mod axo_internal;
mod axo_parser;
mod axo_resolver;
mod axo_scanner;
mod axo_schema;
mod axo_text;

pub use {
    axo_data::*,
    axo_format::*,
    axo_internal::*,
    axo_text::*,
};

use {
    axo_internal::{
        compiler::{
            Compiler,
        },
        logger::{LogInfo, LogPlan, Logger},
        timer::{
            Timer,
        },
    },
    log::Level,
};

#[cfg(target_arch = "x86_64")]
pub const TIMER: timer::CPUCycleSource = timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMER: timer::ARMGenericTimerSource = timer::ARMGenericTimerSource;

pub mod error {
    pub use {
        core::{
            error::Error,
        }
    };
}

pub mod file {
    pub use {
        std::{
            fs::{
                read_to_string,
            },
        },
    };
    //pub use std::io::Error;
}

pub mod io {
    pub use {
        std::{
            io::{
                stdout, Write
            },
        },
    };
}

pub mod environment {
    pub use {
        std::{
            env::args,
        },
    };
}

pub mod thread {
    pub use {
        std::{
            sync::{
                Arc, Mutex
            },
        },
    };
}

pub mod memory {
    pub use {
        core::{
            mem::{
                discriminant, replace
            },
        }
    };
}

pub mod compare {
    pub use {
        core::{
            cmp::{
                Ordering, PartialEq
            },
        }
    };
}

pub mod hash {
    pub use {
        core::{
            hash::{
                Hash, Hasher,
            },
        },
        hashish::HashSet,
    };
}

pub mod character {
    pub use {
        core::{
            char::{
                from_u32, from_u32_unchecked, MAX,
            }
        },
    };

    use super::axo_data::{Number, Str};

    pub fn parse_radix<T: Number>(input: Str, radix: T) -> Option<T> {
        if input.is_empty() {
            return None;
        }

        let radix_u8 = radix.into();

        if radix_u8 < 2 || radix_u8 > 36 {
            return None;
        }

        let mut accumulator = T::default();

        for &byte in input.as_bytes() {
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'z' => byte - b'a' + 10,
                b'A'..=b'Z' => byte - b'A' + 10,
                _ => return None,
            };

            if value >= radix_u8 {
                return None;
            }

            let digit = T::from(value);

            accumulator = accumulator.mul(radix)
                .add(digit);
        }

        Some(accumulator)
    }
}

pub mod reference {}

pub mod any {
    pub use {
        core::{
            any::{
                Any, TypeId,
            },
        }
    };
}

pub mod operations {
    pub use {
        core::{
            ops::{
                Add, Deref, DerefMut, Div, Mul, Neg, Range, Rem, Sub,
            },
        },
    };
}

pub mod architecture {
    pub use {
        core::{
            arch::asm,
        },
    };
}

pub mod marker {
    pub use {
        core::{
            marker::PhantomData,
        },
    };
}

pub mod string {
    pub use {
        core::{
            str::FromStr,
        },
    };
}

pub mod format {
    pub use {
        core::{
            fmt::{
                Debug, Display,
                Formatter, Result,
                Write
            },
        },
    };
}

fn main() {
    let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message]) .with_separator(" ".to_string());

    let logger = Logger::new(Level::max(), plan);
    logger.init().expect("fuck");

    let mut compiler = Compiler::new();

    compiler.compile();
}