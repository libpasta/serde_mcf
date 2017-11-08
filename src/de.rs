use data_encoding::BASE64_NOPAD;
use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, Visitor};

use std::fmt::Display;
use std::str::Split;

use errors::*;

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        ErrorKind::Custom(msg.to_string()).into()
    }
}

/// Deserializer for the MCF format.
pub struct McfDeserializer<'de, I: Iterator<Item = &'de str>>(I);

impl<'de> McfDeserializer<'de, Split<'de, char>> {
    /// Create a new deserializer from a string ref.
    pub fn new(input: &'de str) -> Self {
        let mut iter = input.split('$');
        iter.next();
        McfDeserializer(iter)
    }
}

/// Deserialize the generic type V from a string.
pub fn from_str<'de, V: Deserialize<'de>>(input: &'de str) -> Result<V> {
    V::deserialize(&mut McfDeserializer::new(input))
}

// Macro which will attempt to parse the input value (either self.0 or
// self.0.next()) into whichever type is used. The parsed value can then be
// deserialized by the visitor.
macro_rules! forward_parsable_to_deserialize_any {
    ($($ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$meth(visitor),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        )*
    };
    ($(iter $ty:ident => $meth:ident,)*) => {
        $(
            fn $meth<V>(self, visitor: V) -> Result<V::Value> where V: de::Visitor<'de> {
                if let Some(v) = self.0.next() {
                    match v.parse::<$ty>() {
                        Ok(val) => val.into_deserializer().$meth(visitor),
                        Err(e) => Err(de::Error::custom(e))
                    }
                } else {

                    Err("no value found".into())
                }
            }
        )*
    }
}


impl<'a, 'de, I: Iterator<Item = &'de str>> Deserializer<'de> for &'a mut McfDeserializer<'de, I> {
    type Error = Error;

    // By default attempt to visit a string.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(k) = self.0.next() {
            visitor.visit_borrowed_str(k)
        } else {
            Err("No field to deserialize".into())
        }
    }

    // A struct is deserialized by iterating through the expected fields, and
    // returning each value one-by-one.
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: Visitor<'de>
    {
        // TODO: could change this to visit_seq?
        visitor.visit_map(McfWithFields(self, fields.to_vec().into_iter()))
    }

    // Attempt to deserialize the enum by simply checking the next field for a
    // variant name.
    fn deserialize_enum<V>(self,
                           _name: &'static str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_enum(self)
    }

    // Deserialize the next value as an identifer.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(k) = self.0.next() {
            visitor.visit_borrowed_str(k)
        } else {
            Err("No field to deserialize".into())
        }
    }

    // Deserialize a byte buf by first converting the field from base64.
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(v) = self.0.next() {
            visitor.visit_byte_buf(BASE64_NOPAD.decode(v.as_bytes())?)
        } else {
            Err("no value found".into())
        }
    }

    // A sequence is defined as a list of comma-separated values val1,val2,...
    // We construct a new deserializer which has already split these.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(v) = self.0.next() {
            let iter = v.split(',');
            visitor.visit_seq(&mut McfDeserializer(iter))
        } else {
            Err("no value found".into())
        }
    }

    // Deserializer a tuple by treating it as a sequence.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(v) = self.0.next() {
            let iter = v.split(',');
            visitor.visit_seq(&mut McfDeserializer(iter))
        } else {
            Err("no value found".into())
        }
    }

    // Deserialize a map by splitting on '=' and ',', returning each value one-
    // by-one.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(v) = self.0.next() {
            let iter = v.split(|c| c == '=' || c == ',');
            visitor.visit_map(&mut McfDeserializer(iter))
        } else {
            Err("no value found".into())
        }
    }

    // We consider a None value to be a missing value between two delimiters.
    // Anything else is deserializer as a Some value.
    //
    // This currently only works for flat options.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        if let Some(v) = self.0.next() {
            match v {
                "" => visitor.visit_none(),
                v => visitor.visit_some(&mut McfDeserializer([v].iter().cloned())),
            }
        } else {
            Err("no value found".into())
        }
    }

    forward_to_deserialize_any! {
        char str
        string bytes unit unit_struct newtype_struct
        tuple_struct ignored_any
    }

    forward_parsable_to_deserialize_any! {
        iter bool => deserialize_bool,
        iter u8 => deserialize_u8,
        iter u16 => deserialize_u16,
        iter u32 => deserialize_u32,
        iter u64 => deserialize_u64,
        iter i8 => deserialize_i8,
        iter i16 => deserialize_i16,
        iter i32 => deserialize_i32,
        iter i64 => deserialize_i64,
        iter f32 => deserialize_f32,
        iter f64 => deserialize_f64,
    }
}

// This is used to deserialize any map-like object by forcing the keys to be
// whatever is returned from the iterator J.
struct McfWithFields<'a, 'de: 'a, I: 'a + Iterator<Item=&'de str>, J: Iterator<Item=&'de str>>(&'a mut McfDeserializer<'de, I>, J);

impl<'a, 'de, I: Iterator<Item = &'de str>, J: Iterator<Item = &'de str>> de::MapAccess<'de>
    for
    McfWithFields<'a, 'de, I, J> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed<'de>
    {
        // Take the next field from the iterator and deserialize it.
        if let Some(field) = self.1.next() {
            seed.deserialize(&mut McfDeserializer([field].iter().cloned())).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed<'de>
    {
        // Continue to deserialize from the McfDeserializer
        seed.deserialize(&mut *self.0)
    }
}

impl<'a, 'de, I: Iterator<Item = &'de str>> de::MapAccess<'de> for &'a mut McfDeserializer<'de, I> {
    type Error = Error;

    // Similar to the above, but assumes all values are being returned from a
    // single iterator/deserializer.
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed<'de>
    {
        if let Some(field) = self.0.next() {
            seed.deserialize(&mut McfDeserializer([field].iter().cloned())).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed<'de>
    {
        seed.deserialize(&mut **self)
    }
}


impl<'a, 'de, I: Iterator<Item = &'de str>> de::EnumAccess<'de>
    for &'a mut McfDeserializer<'de, I> {
    type Error = Error;
    type Variant = &'a mut McfDeserializer<'de, I>;

    // Take the next value from the iterator and attept to deserialize it.
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        if let Some(value) = self.0.next() {
            let val = seed.deserialize(&mut McfDeserializer([value].iter().cloned()))?;
            Ok((val, self))
        } else {
            Err(de::Error::custom("Not enough fields"))
        }
    }
}


// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'a, 'de, I: Iterator<Item = &'de str>> de::VariantAccess<'de>
    for
    &'a mut McfDeserializer<'de, I> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        // Err("expected a string".into())
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
        where T: de::DeserializeSeed<'de>
    {
        seed.deserialize(self)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}

impl<'a, 'de, I: Iterator<Item = &'de str>> de::SeqAccess<'de> for &'a mut McfDeserializer<'de, I> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed<'de>
    {
        if let Some(v) = self.0.next() {
            seed.deserialize(&mut McfDeserializer::new(v)).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use serde_bytes;
    use std::collections::HashMap;

    #[test]
    fn test_deserialize() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestStruct {
            p: u8,
            r: Option<u8>,
            params: HashMap<String, String>,
            #[serde(with="serde_bytes")]
            hash: Vec<u8>,
        }

        let mut map = HashMap::new();
        map.insert("x".to_string(), "xylo".to_string());
        map.insert("y".to_string(), "yell".to_string());
        let t = TestStruct {
            p: 12,
            r: Some(5),
            params: map,
            hash: vec![0x12, 0x23, 0x34],
        };

        let ts = "$12$5$x=xylo,y=yell$EiM0";
        assert_eq!(super::from_str::<TestStruct>(ts).unwrap(), t);

        #[derive(Debug, PartialEq, Deserialize)]
        enum TestEnum {
            First { a: u8, b: u8 },
        }

        let t = TestEnum::First { a: 38, b: 128 };

        let ts = "$First$38$128";
        assert_eq!(super::from_str::<TestEnum>(ts).unwrap(), t);
    }
}
