/// Additional methods to deserialize to/from byte arrays encoded in base64.

/// Helper methods for serializing byte arryays to/from base64 encoded format.
pub mod base64 {
    use serde::{Deserialize, Deserializer, Serializer};
    use data_encoding::base64;
    use serde::de::Error;

    pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: AsRef<[u8]>,
              S: Serializer
    {
        serializer.serialize_str(&base64::encode_nopad(bytes.as_ref()))
    }

    pub fn deserialize<'de, T: From<Vec<u8>>, D>(deserializer: D) -> Result<T, D::Error>
        where D: Deserializer<'de>
    {
        String::deserialize(deserializer).map(|s| {
                base64::decode_nopad(s.as_bytes()) // decode from base64
            .map(T::from) // convert to T
            .map_err(|e| Error::custom(e.to_string()))
            })?
    }
}


pub mod base64bcrypt {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;

    use data_encoding::{decode, encode, base};
    use std::marker::PhantomData;

    const X_: u8 = 128;
    /// Force static dispatch.
    enum Static {}
    static BASE: base::Opt<Static> = base::Opt {
        val: &[X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, 0_, 1_, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, X_, X_, X_, X_, X_,
               X_, X_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
               21, 22, 23, 24, 25, 26, 27, X_, X_, X_, X_, X_, X_, 28, 29, 30, 31, 32, 33, 34, 35,
               36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
               X_, X_, X_, X_],
        sym: b"./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
        bit: 6,
        pad: b'=',
        _phantom: PhantomData::<Static>,
    };

    pub fn encode_nopad(input: &[u8]) -> String {
        encode::encode_nopad(&BASE, input)
    }

    pub fn decode_nopad(input: &[u8]) -> Result<Vec<u8>, decode::Error> {
        decode::decode_nopad(&BASE, input).map_err(|e| e.into())
    }

    /// Custom deserialize method for `Bcrypt`.
    pub fn serialize<T, S>(bytes: &(T, T), serializer: S) -> Result<S::Ok, S::Error>
        where T: AsRef<[u8]>,
              S: Serializer
    {
        serializer.serialize_str(
            &(encode_nopad(bytes.0.as_ref()) + 
             &encode_nopad(bytes.1.as_ref()))
        )
    }

    /// Custom deserialize method for `Bcrypt`
    pub fn deserialize<'de, D>(deserializer: D) -> Result<(Vec<u8>, Vec<u8>), D::Error>
        where D: Deserializer<'de>
    {
        let encoded = String::deserialize(deserializer)?;
        let (salt, hash) = (try!(decode_nopad(&encoded.as_bytes()[..22])
                                .map_err(|e| Error::custom(e.to_string()))),
                            try!(decode_nopad(&encoded.as_bytes()[22..])
                                .map_err(|e| Error::custom(e.to_string()))));
        Ok((salt, hash))
    }

    #[test]
    fn check() {
        use data_encoding::base::{Spec, equal, valid};
        const SPEC: Spec = Spec {
            val: &[(b'.', b'/'), (b'A', b'Z'), (b'a', b'z'), (b'0', b'9')],
            pad: b'=',
        };
        assert_eq!(BASE.val.len(), 256);
        assert_eq!(BASE.sym.len(), 1 << BASE.bit);
        valid(&SPEC).unwrap();
        valid(&BASE).unwrap();
        equal(&BASE, &SPEC).unwrap();
    }
}
