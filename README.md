# N85 encoder & decoder.

[![docs.rs](https://docs.rs/n85/badge.svg)](https://docs.rs/n85)
[![crates.io](https://img.shields.io/crates/v/n85.svg)](https://crates.io/crates/n85)
[![crates.io](https://img.shields.io/crates/d/n85.svg)](https://crates.io/crates/n85)

N85 is a binary-to-ASCII encoding based on Ascii85 but more suitable for use as strings
(i.e. excludes `\`, `'` and `"`) and with a simpler implementation than the other variants.

Every 4 bytes is mapped to 5 characters, adding ~25% of storage overhead.
For comparison, base64 maps every 3 bytes to 4 characters, adding ~33% overhead.

## Example

```rust
let s = "Hello, world!";

let enc = n85::encode_string(s.as_ref());
assert_eq!(&enc, "Yb(qJ[NH@N0AO?HI(");

let dec = n85::decode_vec(enc.as_ref()).unwrap();
assert_eq!(&dec, s.as_bytes());
```

## Specification

An arbitrary byte string is split into chunks for 32-bit little endian integers.
The last chunk is padded with zeroes.

Every integer is 5 times divided by 85, giving 5 remainders.
40 (`(`) is added to each remainder.
If the result is equal or greater to 92 (`\`), 1 is added.

If the last chunk is 1, 2, 3 or 4 byte large,
2, 3, 4 or 5 characters are used for encoding respectively.
