use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::mem::swap;
use std::str::FromStr;

use derive_new::new;
use eyre::{Report};
use serde::de::{DeserializeSeed, EnumAccess, Error, Expected, Unexpected, VariantAccess, Visitor};
use serde::Deserialize;
use serde::ser::StdError;

type ArgIn<'a> = (&'a str, Option<&'a str>);

pub fn deserialize_from_args<'de, D, I>(args: I) -> Result<D>
    where D: Deserialize<'de>,
          I: Iterator<Item=ArgIn<'de>>
{
    let mut deserializer = Deserializer::new(args.peekable());
    D::deserialize(&mut deserializer)
}

enum ArgValue<'a> {
    Bool(bool),
    Str(&'a str),
}

struct Arg<'a> {
    name: &'a str,
    value: ArgValue<'a>,
}

#[derive(Debug)]
pub struct SimpleError {
    report: Report,
}

impl SimpleError {
    fn invalid_type_for(name: &str, unexp: Unexpected, exp: &dyn Expected) -> Self {
        Error::custom(format_args!("invalid type for '{name}': {unexp}, expected {exp}"))
    }
}

impl StdError for SimpleError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.report.source()
    }
}

impl Display for SimpleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.report.fmt(f)
    }
}

impl Error for SimpleError {
    fn custom<T: Display>(msg: T) -> Self {
        SimpleError { report: Report::msg(msg.to_string()) }
    }
}

#[derive(new)]
struct Deserializer<'de, I: Iterator<Item=ArgIn<'de>>> {
    args: Peekable<I>,

    #[new(default)]
    next_arg: Option<Arg<'de>>,

    #[new(default)]
    expected_fields: Option<&'static [&'static str]>,

    #[new(default)]
    next_enum_value: Option<&'de str>,
}

type Result<T> = std::result::Result<T, SimpleError>;

impl<'de, I> Deserializer<'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    fn expect_next_arg(&mut self) -> Result<(&'de str, ArgValue<'de>)> {
        SimpleError::custom("Expected a value, but found an identifier");

        // Swap out next_arg
        let mut next_arg: Option<Arg> = None;
        swap(&mut self.next_arg, &mut next_arg);

        next_arg
                .map(|arg| (arg.name, arg.value))
                .ok_or_else(|| Error::custom(
                    "Expected a value, but found an identifier or end of arguments".to_string()
                ))
    }

    fn expect_parse<F: FromStr, V: Visitor<'de>>(&mut self, visitor: &V) -> Result<F> {
        let (name, value) = self.expect_next_arg()?;
        let value = match value {
            ArgValue::Bool(value) =>
                return Err(SimpleError::invalid_type_for(name, Unexpected::Bool(value), visitor)),
            ArgValue::Str(value) => {
                value.parse().map_err(|_| {
                    SimpleError::invalid_type_for(name, Unexpected::Str(value), visitor)
                })?
            }
        };
        Ok(value)
    }

    fn expect_str<V: Visitor<'de>>(&mut self, visitor: &V) -> Result<&'de str> {
        let (name, value) = self.expect_next_arg()?;
        let value = match value {
            ArgValue::Bool(value) =>
                return Err(SimpleError::invalid_type_for(name, Unexpected::Bool(value), visitor)),
            ArgValue::Str(value) => {
                value
            }
        };
        Ok(value)
    }

    fn expect_enum(&mut self, variants: &'static [&'static str])
                   -> Result<(&'de str, &'de str)>
    {
        let (name, value) = self.expect_next_arg()?;
        let value = match value {
            ArgValue::Bool(value) => {
                let all_variants = itertools::intersperse(
                    variants.iter().map(|&v| format!("`{v}`")),
                    ", ".to_string(),
                ).collect::<String>();
                return Err(SimpleError::invalid_type_for(
                    name, Unexpected::Bool(value), &format!("an enum variant, one of {all_variants}").as_str(),
                ));
            }
            ArgValue::Str(value) => {
                value
            }
        };
        Ok((name, value))
    }
}

impl<'a, 'de, I> serde::Deserializer<'de> for &'a mut Deserializer<'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    type Error = SimpleError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let (_, value) = self.expect_next_arg()?;
        match value {
            ArgValue::Bool(value) => visitor.visit_bool(value),
            ArgValue::Str(value) => visitor.visit_borrowed_str(value),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let (name, value) = self.expect_next_arg()?;
        match value {
            ArgValue::Bool(value) => visitor.visit_bool(value),
            ArgValue::Str(value) =>
                Err(SimpleError::invalid_type_for(name, Unexpected::Str(value), &visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_i64(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_u64(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_parse(&visitor)?;
        visitor.visit_char(value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_str(&visitor)?;
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let value = self.expect_str(&visitor)?;
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("binary values are not supported"))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("binary values are not supported"))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("unit values are not supported"))
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("unit struct values are not supported"))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("newtype struct values are not supported"))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("sequence values are not supported"))
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("tuple values are not supported"))
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("tuple struct values are not supported"))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::custom("map values are not supported"))
    }

    fn deserialize_struct<V>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if self.expected_fields.is_some() {
            return Err(Error::custom(
                format!("Already serializing a struct, cannot serialize nested struct {name}"))
            );
        }
        self.expected_fields = Some(fields);
        visitor.visit_map(MapAccessImpl { de: self })
    }

    fn deserialize_enum<V>(mut self, name: &'static str, variants: &'static [&'static str], visitor: V)
                           -> Result<V::Value> where V: Visitor<'de>
    {
        let (field_name, next_enum_value) = self.expect_enum(variants)?;
        self.next_enum_value = Some(next_enum_value);
        visitor.visit_enum(EnumAccessImpl { de: self, type_name: name, field_name })
    }

    fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(next_enum_value) = self.next_enum_value {
            self.next_enum_value = None;
            return visitor.visit_borrowed_str(next_enum_value);
            //return self.deserialize_str(visitor);
        }

        let (name, value) = self.args.next().ok_or_else(|| Error::custom("missing identifier"))?;
        let (name, value) = match value {
            None =>
                if let Some(name_without_prefix) = name.strip_prefix('-') {
                    (name_without_prefix, ArgValue::Bool(false))
                } else {
                    (name, ArgValue::Bool(true))
                },
            Some(str) => {
                (name, ArgValue::Str(str))
            }
        };
        self.next_arg = Some(Arg { name, value });
        visitor.visit_borrowed_str(name)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let (name, _) = self.expect_next_arg()?;
        Err(SimpleError::unknown_field(name, self.expected_fields.unwrap_or(&[])))
    }
}

struct MapAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    de: &'a mut Deserializer<'de, I>,
}

impl<'a, 'de, I> serde::de::MapAccess<'de> for MapAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    type Error = SimpleError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: DeserializeSeed<'de>
    {
        if self.de.args.peek().is_none() {
            return Ok(None); // No more elements in map
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: DeserializeSeed<'de>
    {
        seed.deserialize(&mut *self.de)
    }
}

struct EnumAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>,
{
    type_name: &'a str,
    field_name: &'a str,
    de: &'a mut Deserializer<'de, I>,
}

impl<'a, 'de, I> EnumAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>,
{
    fn unsupported_variant(&self) -> SimpleError {
        let type_name = self.type_name;
        let field_name = self.field_name;
        SimpleError::custom(format!("Non-unit enum variants are not supported (enum type {type_name}, field {field_name}"))
    }
}

impl<'a, 'de, I> EnumAccess<'de> for EnumAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    type Error = SimpleError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)> where V: DeserializeSeed<'de> {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'a, 'de, I> VariantAccess<'de> for EnumAccessImpl<'a, 'de, I>
    where I: Iterator<Item=ArgIn<'de>>
{
    type Error = SimpleError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value> where T: DeserializeSeed<'de> {
        Err(self.unsupported_variant())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(self.unsupported_variant())
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(self.unsupported_variant())
    }
}
