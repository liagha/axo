use {
    crate::{
        format::{
            self,
            Display,
            Formatter,
        },
    },
};

pub enum ErrorKind {
    
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "") 
    }
}