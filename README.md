[MaybeUtf8][doc] 0.2.2
======================

[![MaybeUTF8 on Travis CI][travis-image]][travis]

[travis-image]: https://travis-ci.org/lifthrasiir/rust-maybe_utf8.png
[travis]: https://travis-ci.org/lifthrasiir/rust-maybe_utf8

Byte container optionally encoded as UTF-8.
It is intended as a byte sequence type with uncertain character encoding,
while the caller might be able to determine the actual encoding.

For example, [ZIP file format](https://en.wikipedia.org/wiki/Zip_%28file_format%29)
originally didn't support UTF-8 file names,
assuming the archive would be extracted only in the system
with the same system encoding as the original system.
The newer ZIP standard supports explicitly UTF-8-encoded file names though.
In this case, the ZIP library may want to return either a `String` or `Vec<u8>`
depending on the UTF-8 flag.

This crate supports two types,
`MaybeUtf8Buf` (analogous to `String`) and `MaybeUtf8Slice` (analogous to `&str`).
Both types support various conversion methods.
For example, if you know that the bytes are encoded in ISO 8859-2,
[Encoding](https://github.com/lifthrasiir/rust-encoding/) can be used to convert them:

```rust
use std::borrow::IntoCow;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_2;
use maybe_utf8::{MaybeUtf8Buf, MaybeUtf8Slice};

let namebuf = MaybeUtf8Buf::from_bytes(vec![99,97,102,233]);
assert_eq!(format!("{}", namebuf), "caf\u{fffd}");

// borrowed slice equally works
{
    let nameslice: MaybeUtf8Slice = namebuf.to_slice();
    assert_eq!(format!("{:?}", nameslice), r#"b"caf\xe9""#);
    assert_eq!(nameslice.map_as_cow(|v| ISO_8859_2.decode(&v, DecoderTrap::Replace).unwrap()),
               "caf\u{e9}");
}

// consuming an optionally-UTF-8-encoded buffer also works
assert_eq!(namebuf.map_into_str(|v| ISO_8859_2.decode(&v, DecoderTrap::Replace).unwrap()),
           "caf\u{e9}");
```

`IntoMaybeUtf8` trait can be used to uniformly accept either string or vector
to construct `MaybeUtf8*` values.

```rust
use maybe_utf8::IntoMaybeUtf8;
assert_eq!("caf\u{e9}".into_maybe_utf8(), b"caf\xc3\xa9".into_maybe_utf8());
```

[Complete Documentation][doc] is available.

MaybeUtf8 is written by Kang Seonghoon and licensed under the MIT/X11 license.

[doc]: https://lifthrasiir.github.io/rust-maybe_utf8/

