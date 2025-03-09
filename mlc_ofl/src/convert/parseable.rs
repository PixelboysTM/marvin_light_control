use serde_json::{Map, Value};
use mlc_data::{err, ContextResult, misc::ContextError};

pub trait Parseable: Sized {
    fn parse_from_value(value: &Value) -> ContextResult<Self>;
    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self>;
}

pub trait SimpleParseable: Sized {
    fn parse_from_value(value: &Value) -> ContextResult<Self>;
    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        let v = obj.get(key).ok_or(err!("'{}' must be present in object", key))?;
        Self::parse_from_value(v)
    }
}

pub trait SimpleParseableMarker {}

impl<T: SimpleParseable> SimpleParseableMarker for T {}

impl<T: SimpleParseable> Parseable for T {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        SimpleParseable::parse_from_value(value)
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        SimpleParseable::parse_from_object(obj, key)
    }
}