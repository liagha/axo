use {
    crate::{
        format::{Display},  
    },
};

pub enum ErrorKind {

}

impl Display for ErrorKind {
    fn fmt(&self, _f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {}
    }
}