# GSM-7 (aka GSM 03.38 or 3GPP 23.038) encoding and decoding in Rust

[API Documentation](https://docs.rs/gsm7)

# Example

## Decoding

```rust
let v = vec![84, 58, 157, 14];
let reader = Gsm7Reader::new(io::Cursor::new(&v));
let s = reader.collect::<io::Result<String>>()?;
assert_eq!(&s, "Tttt");
```

## Encoding

```rust
let mut writer = Gsm7Writer::new(Vec::new());
writer.write_str("Hello")?;

let v = writer.into_writer()?;
println!("v: {:?}", v);
```

# License
gsm7 is distributed under the MIT license.

