use errors::*;
use std::iter::{Filter, Peekable};
use std::str::{FromStr, Split};

use serde::de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};

pub struct Deserializer<'de, F>
where
    F: FnMut(char) -> bool,
{
    iter: Peekable<Filter<Split<'de, F>, fn(&&str) -> bool>>,
}

impl<'de, F> Deserializer<'de, F>
where
    F: FnMut(char) -> bool,
{
    pub fn from_str(s: &'de str) -> Deserializer<impl FnMut(char) -> bool> {
        fn is_not_empty(s: &&str) -> bool {
            !s.is_empty()
        }
        let is_not_empty = is_not_empty as fn(&&str) -> bool;

        fn is_whitespace(c: char) -> bool {
            c.is_whitespace()
        }

        Deserializer {
            iter: s.split(is_whitespace).filter(is_not_empty).peekable(),
        }
    }

    pub fn from_closure(f: F, s: &'de str) -> Self {
        fn is_not_empty(s: &&str) -> bool {
            !s.is_empty()
        }
        let is_not_empty = is_not_empty as fn(&&str) -> bool;

        Deserializer {
            iter: s.split(f).filter(is_not_empty).peekable(),
        }
    }

    fn parse_next<T: FromStr>(&mut self) -> Result<T, ScanError> {
        match self.iter.next() {
            Some(s) => s.parse().map_err(|_| ScanError::De),
            None => Err(ScanError::EOF),
        }
    }

    fn next(&mut self) -> Result<&'de str, ScanError> {
        self.iter.next().ok_or(ScanError::EOF)
    }

    fn peek(&mut self) -> Option<&&'de str> {
        self.iter.peek()
    }
}

enum NextValue {
    Unsigned,
    Integer,
    Float,
    Char,
    String,
}

impl NextValue {
    fn new(next: &str) -> Self {
        if next.parse::<u64>().is_ok() {
            NextValue::Unsigned
        } else if next.parse::<i64>().is_ok() {
            NextValue::Integer
        } else if next.parse::<f64>().is_ok() {
            NextValue::Float
        } else if next.parse::<char>().is_ok() {
            NextValue::Char
        } else {
            NextValue::String
        }
    }
}

impl<'de, 'a, F> de::Deserializer<'de> for &'a mut Deserializer<'de, F>
where
    F: FnMut(char) -> bool,
{
    type Error = ScanError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(next) = self.peek().map(|s| NextValue::new(*s)) {
            match next {
                NextValue::Float => self.deserialize_f64(visitor),
                NextValue::Integer => self.deserialize_i64(visitor),
                NextValue::Unsigned => self.deserialize_u64(visitor),
                NextValue::Char => self.deserialize_char(visitor),
                _ => self.deserialize_str(visitor),
            }
        } else {
            Err(ScanError::EOF)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_next()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_next()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_next()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_next()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_next()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_next()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_next()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_next()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_next()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_next()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_next()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.parse_next()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.next()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // borrowed bytestrings not supported
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // treat Vec<u8> like a regular vec
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // TODO: better option parsing
        if self.peek().is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Sequence::new(&mut *self))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Sequence::new(&mut *self).with_limit(len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Sequence::new(&mut *self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Sequence::new(&mut *self).with_names(variants))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(Sequence::new(&mut *self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // like any, not possible
        Err(ScanError::NS("self describing formats"))
    }
}

struct Sequence<'de, 'a, F>
where
    'de: 'a,
    F: FnMut(char) -> bool,
{
    de: &'a mut Deserializer<'de, F>,
    count: usize,
    names: Option<&'a [&'static str]>,
    limit: Option<usize>,
}

impl<'de, 'a, F> Sequence<'de, 'a, F>
where
    F: FnMut(char) -> bool,
{
    fn new(de: &'a mut Deserializer<'de, F>) -> Self {
        Sequence {
            de,
            count: 0,
            names: None,
            limit: None,
        }
    }

    fn with_names(self, names: &'a [&'static str]) -> Self {
        let mut new = self;
        new.names = Some(names);
        new
    }

    fn with_limit(self, limit: usize) -> Self {
        let mut new = self;
        new.limit = Some(limit);
        new
    }
}

impl<'de, 'a, F> SeqAccess<'de> for Sequence<'de, 'a, F>
where
    F: FnMut(char) -> bool,
{
    type Error = ScanError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(lim) = self.limit {
            // if we have a limit defined, stop
            if lim == self.count {
                return Ok(None);
            }
        }

        if let None = self.de.peek() {
            // if we have no more data, stop
            return Ok(None);
        }

        self.count += 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, 'a, F> MapAccess<'de> for Sequence<'de, 'a, F>
where
    F: FnMut(char) -> bool,
{
    type Error = ScanError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        use serde::de::IntoDeserializer;

        // if we have the names, use them
        if let Some(names) = self.names {
            if self.count >= names.len() {
                return Ok(None);
            } else {
                self.count += 1;
                return seed
                    .deserialize(names[self.count - 1].into_deserializer())
                    .map(Some);
            }
        }

        // if theres nothing left, return none
        if let None = self.de.peek() {
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

impl<'de, 'a, F> EnumAccess<'de> for Sequence<'de, 'a, F>
where
    F: FnMut(char) -> bool,
{
    type Error = ScanError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de).map(|v| (v, self))
    }
}

impl<'de, 'a, F> VariantAccess<'de> for Sequence<'de, 'a, F>
where
    F: FnMut(char) -> bool,
{
    type Error = ScanError;

    // unit should be caught by EnumAccess,
    // newtype, tuple, and struct variants not supported atm
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(ScanError::NS("struct enum variants"))
    }
}
