use crate::convert::parseable::Parseable;
use mlc_data::ContextResult;
use serde_json::{Map, Value};

pub trait ParseableDefault: Sized {
    fn parse_from_object_default(obj: &Map<String, Value>, key: &str, default: Self) -> ContextResult<Self>;
    fn parse_from_value_default(value: &Value, default: Self) -> ContextResult<Self>;
}

impl<T> ParseableDefault for T where Option<T>: Parseable {
    fn parse_from_object_default(obj: &Map<String, Value>, key: &str, default: Self) -> ContextResult<Self> {
        Ok(obj.parse::<Option<Self<>>>(key)?.unwrap_or(default))
    }
    fn parse_from_value_default(value: &Value, default: Self) -> ContextResult<Self> {
        Ok(value.parse::<Option<Self>>()?.unwrap_or(default))
    }
}

pub trait ParseExecutorObj {
    fn parse<T>(&self, key: &str) -> ContextResult<T> where T: Parseable;
    fn parse_default<T>(&self, key: &str, default: T) -> ContextResult<T> where Option<T>: Parseable;
}

impl ParseExecutorObj for Map<String, Value> {
    fn parse<T>(&self, key: &str) -> ContextResult<T> where T: Parseable     {
        T::parse_from_object(self, key)
    }
    fn parse_default<T>(&self, key: &str, default: T) -> ContextResult<T> where Option<T>: Parseable{
        T::parse_from_object_default(self, key, default)
    }
}

pub trait ParseExecutorValue {
    fn parse<T>(&self) -> ContextResult<T> where T: Parseable;
    fn parse_default<T>(&self, default: T) -> ContextResult<T> where Option<T>: Parseable;
}

impl ParseExecutorValue for Value {
    fn parse<T>(&self) -> ContextResult<T> where T: Parseable {
        T::parse_from_value(self)
    }
    fn parse_default<T>(&self, default: T) -> ContextResult<T> where Option<T>: Parseable {
        T::parse_from_value_default(self, default)
    }
}