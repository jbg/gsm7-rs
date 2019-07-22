# GSM-7 (aka GSM 03.38 or 3GPP 23.038) encoding and decoding in Rust

[API Documentation](https://docs.rs/gsm7)

# Example

## Decoding

```rust
let v: Vec<_> = vec![84, 58, 157, 14].into_iter().collect();
let reader = Gsm7Reader::new(io::Cursor::new(&v));
let s: String = reader.collect::<io::Result<_>>().unwrap();
assert_eq!(&s, "Tttt");
```

## Encoding

```rust
let mut writer = Gsm7Writer::new(Vec::new());
writer.write_char('H').unwrap();
writer.write_char('e').unwrap();
writer.write_char('l').unwrap();
writer.write_char('l').unwrap();
writer.write_char('o').unwrap();

let v = writer.into_writer().unwrap();
println!("v: {:?}", v);
```

# License
gsm7 is distributed under the MIT license.

