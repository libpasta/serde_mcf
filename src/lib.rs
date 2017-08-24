/// Serde functionality for the `ModularCryptFormat`.
///
/// This is informally defined in the following way:
///
/// Fields are delimited by $ signs, and are simply decoded in order.
/// So the struct Foo { x: 12, y: 37} serializers to/from the string `$12$37`.
///
/// Fields can either be `UnitVariants`, and decode by name, single values,
/// or Maps in the form key=value,...,. Finally, a field can also contain a
/// byte array, which by default serializes to a base64 string, unpadded.

extern crate data_encoding;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod de;
pub use de::{from_str, McfDeserializer};

mod encoding;
pub use encoding::base64;
pub use encoding::base64bcrypt;

pub mod ser;
pub use ser::{to_string, McfSerializer};

pub use serde_json::{Map, Value};

/// A generic hash converted from the `ModularCryptFormat`.
///
/// 
#[derive(Debug, Deserialize, Serialize)]
pub struct McfHash {
    pub algorithm: Hashes,
    pub parameters: Map<String, Value>,
    #[serde(with = "base64")]
    pub salt: Vec<u8>,
    #[serde(with = "base64")]
    pub hash: Vec<u8>,
}

pub mod legacy {
    use super::*;
    /// MCF style `Bcrypt` hash
    #[derive(Debug, Deserialize, Serialize)]
    pub struct BcryptHash {
        algorithm: Hashes,
        cost: u8,
        #[serde(with = "base64bcrypt")]
        salthash: (Vec<u8>, Vec<u8>)
    }

    impl Into<McfHash> for BcryptHash {
        fn into(self) -> McfHash {
            let mut params = Map::<String, Value>::new();
            params.insert("cost".to_string(), Value::Number(self.cost.into()));
            McfHash {
                algorithm: self.algorithm,
                parameters: params,
                salt: self.salthash.0,
                hash: self.salthash.1,
            }
        }
    }
}

macro_rules! enum_hashes {
    ($($hash:ident = $val:expr,)*) => (
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        pub enum Hashes {
            $(
            #[serde(rename = $val)]
            $hash,
            )*
        }

        impl Hashes {
            pub fn from_id(id: &str) -> Option<Hashes> {
                match id {
                    $(
                        $val => Some(Hashes::$hash),
                    )*
                    _ => None
                }
            }

            pub fn to_id(&self) -> &'static str {
                match *self {
                    $(
                        Hashes::$hash => $val,
                    )*
                }
            }
        }
    )
}

/// List of known algorithm identifiers.
/// Source: https://passlib.readthedocs.io/en/stable/modular_crypt_format.html
enum_hashes!{
    Md5Crypt = "1",
    Bcrypt = "2",
    Bcrypta = "2a",
    Bcryptx = "2x",
    Bcrypty = "2y",
    Bcryptb = "2b",
    BcryptMcf = "2y-mcf",
    BsdNtHash = "3",
    Sha256Crypt = "5",
    Sha512Crypt = "6",
    SunMd5Crypt = "md5",
    Sha1Crypt = "sha1",
    AprMd5Crypt = "apr1", // Apache htdigest files
    Argon2i = "argon2i",
    Argon2d = "argon2d",
    BcryptSha256 = "bcrypt-sha256", // Passlib-specific
    Phpassp = "P", // PHPass-based applicatoins
    Phpassh = "H", // PHPass-based applicatoins
    Pbkdf2Sha1 = "pbkdf2", // Passlib-specific
    Pbkdf2Sha256 = "pbkdf2-sha256", // Passlib-specific
    Pbkdf2Sha512 = "pbkdf2-sha512", // Passlib-specific
    Scram = "scram", // Passlib-specific
    CtaPbkdf2Sha1 = "p5k2",
    Scrypt = "scrypt",  // Passlib-specific
    ScryptMcf = "scrypt-mcf",

    Hmac = "hmac", // for libpasta
    Custom = "custom", // for any other purposes. fill details in params field
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_all() {
        let argon_hash = "$argon2i$m=262144,p=1,t=2$c29tZXNhbHQ\
                          $Pmiaqj0op3zyvHKlGsUxZnYXURgvHuKS4/Z3p9pMJGc";
        let bcrypt_hash = "$2a$10$ckjEeyTD6estWyoofn4EROM9Ik2PqVcfcrepX.uGp6.aqRdCMN/Oe";

        let argon: McfHash = from_str(argon_hash).unwrap();
        println!("{:?}", argon);
        println!("In JSON: {}", serde_json::to_string_pretty(&argon).unwrap());
        assert_eq!(to_string(&argon).unwrap(), argon_hash);
        let bcrypt: legacy::BcryptHash = from_str(bcrypt_hash).unwrap();
        println!("{:?}", bcrypt);
        println!("In JSON: {}", serde_json::to_string_pretty(&bcrypt).unwrap());
        let updated: McfHash = bcrypt.into();
        println!("In JSON: {}", serde_json::to_string_pretty(&updated).unwrap());
        // assert_eq!(to_string(updated).unwrap(), bcrypt_hash);

    }

    #[test]
    fn test_trial_deserialize() {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum BcryptOrArgon {
            Argon(McfHash),
            Bcrypt(legacy::BcryptHash),
        }

        let argon_hash = "$argon2i$m=262144,p=1,t=2$c29tZXNhbHQ\
                          $Pmiaqj0op3zyvHKlGsUxZnYXURgvHuKS4/Z3p9pMJGc";
        let bcrypt_hash = "$2a$10$ckjEeyTD6estWyoofn4EROM9Ik2PqVcfcrepX.uGp6.aqRdCMN/Oe";

        let argon = {
            match from_str::<McfHash>(argon_hash) {
                Ok(v) => BcryptOrArgon::Argon(v),
                Err(_) => {
                    let v = from_str::<legacy::BcryptHash>(argon_hash).unwrap();
                    BcryptOrArgon::Bcrypt(v)
                }
            }
        };



        assert!(if let BcryptOrArgon::Argon(_) = argon { true} else { false });

        let bcrypt = {
            match from_str::<McfHash>(bcrypt_hash) {
                Ok(v) => BcryptOrArgon::Argon(v),
                Err(_) => {
                    let v = from_str::<legacy::BcryptHash>(bcrypt_hash).unwrap();
                    BcryptOrArgon::Bcrypt(v)
                }
            }
        };

        assert!(if let BcryptOrArgon::Bcrypt(_) = bcrypt { true} else { false });
    }
}
