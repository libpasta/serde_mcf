Serde MCF
===================================

This is a Rust library for a [serde](https://serde.rs/) deserializer/serializer
for the modular crypt format (MCF).

MCF was slightly more formally defined in the [password hashing competition](https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md)
which we use as a rough guide for this library.

While this can be used as a general-purpose format this is not recommended,
and should only be used for serializing password hashes.

Installation
============

This crate works with Cargo and can be found on
[crates.io] with a `Cargo.toml` like:

```toml
[dependencies]
serde_mcf = "0.1.0"
```

[crates.io]: https://crates.io/crates/serde_mcf

## License

serde_mcf is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in serde_qs by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
