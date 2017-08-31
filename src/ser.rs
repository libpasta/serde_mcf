use serde::ser::*;
use serde::ser;
use std::fmt::Display;
use data_encoding;
use std::io::{self, Write};

error_chain!{
    errors { 
        Custom(msg: String)
        Unsupported
    }

    foreign_links {
        Decoding(data_encoding::decode::Error);
        Io(io::Error);
    }
}

/// Serializer for producing MCF-style hashes.
pub struct McfSerializer<W: Write>(W);

impl<'a, W: Write> McfSerializer<W> {
    fn new(writer: W) -> Self {
        McfSerializer(writer)
    }

    fn write<T: AsRef<[u8]>>(&mut self, input: T) -> Result<()> {
        self.0.write_all(input.as_ref()).map_err(|e| e.into())
    }
}

/// Serialize object to a MCF-style hash.
pub fn to_string<S: Serialize>(s: &S) -> Result<String> {
    let mut buf = Vec::new();
    buf.write_all(b"$")?;
    s.serialize(&mut McfSerializer::new(&mut buf))?;
    Ok(String::from_utf8(buf).unwrap())
}

macro_rules! serialize_as_string {
    (mcf $($ty:ty => $meth:ident,)*) => {
        $(
            fn $meth(self, v: $ty) -> Result<Self::Ok> {
                self.write(&v.to_string())
            }
        )*
    };
    ($($ty:ty => $meth:ident,)*) => {
        $(
            fn $meth(self, v: $ty) -> Result<Self::Ok> {
                // Ok(v.to_string())
                Ok(v.to_string())
            }
        )*
    };
}

impl<'a, W: Write> Serializer for &'a mut McfSerializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = McfSeq<'a, W>;
    type SerializeTuple = McfSeq<'a, W>;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = McfSeq<'a, W>;
    type SerializeStruct = McfSeq<'a, W>;
    type SerializeStructVariant = Self;

    serialize_as_string!{
        mcf
        bool => serialize_bool,
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
    }


    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        super::base64::serialize(&value, self)
    }

    /// Returns an error.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              variant: &'static str)
                              -> Result<Self::Ok> {
        self.write(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(self,
                                                            _name: &'static str,
                                                            value: &T)
                                                            -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(self,
                                                             _name: &'static str,
                                                             _variant_index: u32,
                                                             variant: &'static str,
                                                             value: &T)
                                                             -> Result<Self::Ok> {
        self.write(variant)?;
        value.serialize(self)
    }

    /// Returns an error.
    fn serialize_none(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(McfSeq(self, false))
    }


    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(McfSeq(self, false))
    }

    /// Returns an error.
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        self.write(variant)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(McfSeq(self, false))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(McfSeq(self, false))
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        self.write(variant)?;
        Ok(self)
    }
}


impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
        where T: Display
    {
        ErrorKind::Custom(msg.to_string()).into()
    }
}

pub struct McfSeq<'a, W: 'a + Write>(&'a mut McfSerializer<W>, bool);
impl<'a, W: Write> SerializeTuple for McfSeq<'a, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        if self.1 {
            self.0.write(",")?;
        }
        self.1 = true;
        self.0.write(value.serialize(StringSerializer)?)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())

    }
}

impl<'a, W: Write> SerializeSeq for McfSeq<'a, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        if self.1 {
            self.0.write(",")?;
        }
        self.1 = true;
        self.0.write(value.serialize(StringSerializer)?)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())

    }
}

impl<'a, W: Write> SerializeStruct for McfSeq<'a, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        if self.1 {
            self.0.write("$")?;
        }
        self.1 = true;
        value.serialize(&mut *self.0)
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeStructVariant for &'a mut McfSerializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: Serialize
    {
        self.write("$")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleVariant for &'a mut McfSerializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        self.write("$")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleStruct for &'a mut McfSerializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        self.write(value.serialize(StringSerializer)?)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeMap for McfSeq<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
        where T: Serialize
    {
        if self.1 {
            self.0.write(",")?;
        }
        self.1 = true;
        self.0.write(key.serialize(StringSerializer)?)?;
        self.0.write("=")
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: Serialize
    {
        self.0.write(value.serialize(StringSerializer)?)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
        where K: Serialize,
              V: Serialize
    {
        if self.1 {
            self.0.write(",")?;
        }
        self.0.write(key.serialize(StringSerializer)?)?;
        self.0.write("=")?;
        self.1 = true;
        self.0.write(value.serialize(StringSerializer)?)
    }
}


#[cfg(test)]
mod test {
    use serde_bytes;

    #[test]
    fn test_serialize() {
        #[derive(Serialize)]
        struct TestStruct {
            p: u8,
            r: u8,
            #[serde(with="serde_bytes")]
            hash: [u8; 3],
        }

        let t = TestStruct {
            p: 12,
            r: 5,
            hash: [0x12, 0x23, 0x34],
        };

        let ts = super::to_string(&t).unwrap();
        assert_eq!(ts, "$12$5$EiM0");


        #[derive(Serialize)]
        #[serde(tag = "variant")]
        enum TestEnum {
            First { a: u8, b: u8 },
        }

        let t = TestEnum::First { a: 38, b: 128 };

        let ts = super::to_string(&t).unwrap();
        assert_eq!(ts, "$First$38$128");
    }
}

struct StringSerializer;

impl Serializer for StringSerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    serialize_as_string!{
        bool => serialize_bool,
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
    }


    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        super::encoding::base64::serialize(&value, self)
    }

    /// Returns an error.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                              -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(self,
                                                            _name: &'static str,
                                                            _value: &T)
                                                            -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(self,
                                                             _name: &'static str,
                                                             _variant_index: u32,
                                                             _variant: &'static str,
                                                             _value: &T)
                                                             -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_none(self) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(ErrorKind::Unsupported.into())
    }


    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(ErrorKind::Unsupported.into())
    }

    /// Returns an error.
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(ErrorKind::Unsupported.into())

    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(ErrorKind::Unsupported.into())
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        Err(ErrorKind::Unsupported.into())
    }
}
