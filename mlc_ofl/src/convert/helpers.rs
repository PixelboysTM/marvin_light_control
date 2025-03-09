use crate::convert::Parseable;
use crate::convert::parse_helpers::{ParseExecutorObj, ParseExecutorValue};
use either::Either;
use mlc_data::{ContextResult, MaybeLinear, err, misc::ContextError};
use serde_json::{Map, Value};
use std::fmt::Debug;

impl Parseable for bool {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value.as_bool().ok_or(err!("Value must be a bool"))
    }
}

impl Parseable for String {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value
            .as_str()
            .ok_or(err!("Value must be a string"))
            .map(|s| s.to_string())
    }
}

impl Parseable for f32 {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        value
            .as_f64()
            .ok_or(err!("Value must be a float"))
            .map(|f| f as f32)
    }
}

impl Parseable for Value {
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        Ok(value.clone())
    }
}

impl<T> Parseable for Option<T>
where
    T: Parseable,
{
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        if value.is_null() {
            Ok(None)
        } else {
            Some(T::parse_from_value(value)).transpose()
        }
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        if obj.contains_key(key) {
            Some(T::parse_from_value(&obj[key])).transpose()
        } else {
            Ok(None)
        }
    }
}

impl<T> Parseable for Option<MaybeLinear<T>>
where
    T: Parseable + Debug + Clone,
{
    fn parse_from_value(_: &Value) -> ContextResult<Self> {
        Err(err!(
            "MaybeLinear can't parsed from a single value, must be an object"
        ))
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        if let Some(obj) = obj.get(key) {
            Ok(Some(MaybeLinear::Constant(T::parse_from_value(obj)?)))
        } else if let Some(s_obj) = obj.get(&format!("{}Start", key)) {
            let start = T::parse_from_value(s_obj)?;
            let end = T::parse_from_value(obj.get(&format!("{}End", key)).ok_or(err!(
                "if Start is present also End must be there. Key: {key}, Obj: {obj:?}"
            ))?)?;
            Ok(Some(MaybeLinear::Linear { start, end }))
        } else {
            Ok(None)
        }
    }
}

pub trait CustomOptionalParser: Sized {
    type Out;
    fn require(self) -> ContextResult<Self::Out>;
    fn require_default(self, default: Self::Out) -> ContextResult<Self::Out> {
        Ok(self.require().unwrap_or(default))
    }
}

impl<T> CustomOptionalParser for ContextResult<Option<MaybeLinear<T>>>
where
    T: Parseable + Debug + Clone,
{
    type Out = MaybeLinear<T>;

    fn require(self) -> ContextResult<Self::Out> {
        match self? {
            Some(v) => Ok(v),
            None => Err(err!("MaybeLinear is required to be Some")),
        }
    }
}



impl<T> Parseable for Vec<T>
where
    T: Parseable,
{
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let v = value.as_array().ok_or(err!("must be an array"))?;
        v.iter()
            .map(T::parse_from_value)
            .collect::<ContextResult<Vec<T>>>()
    }
}

/// Both types are being parsed th one that succeeds is returned, if both succeed the left one is returned, if both fail an error is returned.
impl<L, R> Parseable for Option<Either<L, R>>
where
    L: Parseable,
    R: Parseable,
{
    fn parse_from_value(value: &Value) -> ContextResult<Self> {
        let left: ContextResult<L> = value.parse();
        let right: ContextResult<R> = value.parse();
        decide(left, right)
    }

    fn parse_from_object(obj: &Map<String, Value>, key: &str) -> ContextResult<Self> {
        let split = key.split(' ').collect::<Vec<_>>();
        if split.len() != 2 {
            return Err(err!(
                "key for Either must be a whitespace seperated list of two values ('<leftKey> <rightKey>') got: '{}'",
                key
            ));
        }

        let left: ContextResult<L> = obj.parse(split[0]);
        let right: ContextResult<R> = obj.parse(split[1]);

        decide(left, right)
    }
}

fn decide<L, R>(
    left: ContextResult<L>,
    right: ContextResult<R>,
) -> ContextResult<Option<Either<L, R>>> {
    match (left, right) {
        (Ok(l), _) => Ok(Some(Either::Left(l))),
        (Err(_), Ok(r)) => Ok(Some(Either::Right(r))),
        _ => Ok(None),
    }
}

impl<L, R> CustomOptionalParser for ContextResult<Option<Either<L, R>>>
where
    L: Parseable,
    R: Parseable,
{
    type Out = Either<L, R>;

    fn require(self) -> ContextResult<Self::Out> {
        self?.ok_or(err!(
            "Either is required but none of the values could be parsed"
        ))
    }
}
