[MaybeUTF8][doc] 0.1.3
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

`MaybeUTF8` type supports various conversion methods.
For example, if you know that the bytes are encoded in ISO 8859-2,
[Encoding](https://github.com/lifthrasiir/rust-encoding/) can be used to convert them:

```rust
extern crate encoding;
use std::borrow::IntoCow;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_2;

let namebuf = MaybeUTF8::from_vec(vec![99,97,102,233]);
let name = namebuf.map_into_str(|v| ISO_8859_2.decode(&*v, DecoderTrap::Replace).unwrap());
assert_eq!(name, "caf\u{e9}");
```

[Complete Documentation][doc] is available.

MaybeUTF8 is written by Kang Seonghoon and licensed under the MIT/X11 license.

[doc]: https://lifthrasiir.github.io/rust-maybe_utf8/

