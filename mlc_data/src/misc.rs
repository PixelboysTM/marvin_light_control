use std::fmt::{Debug, Formatter};
use log::debug;
use crate::DynamicError;

#[macro_export]
macro_rules! err {
    ($arg:expr) => {
        ContextError{
            filename: file!(),
            line: line!(),
            error: $arg.into(),
        }
    };
    ($($arg:tt)+) => {
        ContextError{
            filename: file!(),
            line: line!(),
            error: format!($($arg)+).into(),
        }
    };
}

pub struct ContextError{
    pub filename: &'static str,
    pub line: u32,
    pub error: Box<dyn std::error::Error>,
}

impl Debug for ContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[{}:{}] {}", self.filename, self.line, self.error))
    }
}

impl ContextError {
    pub fn to_generic(self) -> DynamicError {
        format!("{self:?}").into()
    }
}

pub trait ErrIgnore{
    fn ignore(self);
    fn bin(self) where Self: std::marker::Sized {
        self.ignore();
    }
}

impl<T, E> ErrIgnore for Result<T, E> {
    fn ignore(self) {}
}