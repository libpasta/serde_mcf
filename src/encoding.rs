/// Additional methods to deserialize to/from byte arrays encoded in base64.

/// Helper methods for serializing byte arryays to/from base64 encoded format.
pub mod base64 {
    use serde::{Deserialize, Deserializer, Serializer};
    use data_encoding::BASE64_NOPAD;
    use serde::de::Error;

    pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: AsRef<[u8]>,
              S: Serializer
    {
        serializer.serialize_str(&BASE64_NOPAD.encode(bytes.as_ref()))
    }

    pub fn deserialize<'de, T: From<Vec<u8>>, D>(deserializer: D) -> Result<T, D::Error>
        where D: Deserializer<'de>
    {
        String::deserialize(deserializer).map(|s| {
                BASE64_NOPAD.decode(s.as_bytes()) // decode from base64
            .map(T::from) // convert to T
            .map_err(|e| Error::custom(e.to_string()))
            })?
    }
}


pub mod base64bcrypt {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;

    use data_encoding::{Encoding, Specification};

    lazy_static! {
        /// BCrypt-specific base64 encoding scheme.
        static ref BASE64BCRYPT: Encoding = {
            let mut spec = Specification::new();
            spec.symbols.push_str(
                "./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
            spec.encoding().unwrap()
        };
    }

    /// Custom deserialize method for `Bcrypt`.
    pub fn serialize<T, S>(bytes: &(T, T), serializer: S) -> Result<S::Ok, S::Error>
        where T: AsRef<[u8]>,
              S: Serializer
    {
        serializer.serialize_str(
            &(BASE64BCRYPT.encode(bytes.0.as_ref()) + 
             &BASE64BCRYPT.encode(bytes.1.as_ref()))
        )
    }

    /// Custom deserialize method for `Bcrypt`
    pub fn deserialize<'de, D>(deserializer: D) -> Result<(Vec<u8>, Vec<u8>), D::Error>
        where D: Deserializer<'de>
    {
        let encoded = String::deserialize(deserializer)?;
        let (salt, hash) = (try!(BASE64BCRYPT.decode(&encoded.as_bytes()[..22])
                                .map_err(|e| Error::custom(e.to_string()))),
                            try!(BASE64BCRYPT.decode(&encoded.as_bytes()[22..])
                                .map_err(|e| Error::custom(e.to_string()))));
        Ok((salt, hash))
    }
}
