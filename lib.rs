// maybe_utf8: Byte container optionally encoded as UTF-8.
// Copyright (c) 2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!

# MaybeUtf8 0.2.3

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
# extern crate encoding;
# extern crate maybe_utf8;
# fn main() {
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
# }
```

`IntoMaybeUtf8` trait can be used to uniformly accept either string or vector
to construct `MaybeUtf8*` values.

```rust
use maybe_utf8::IntoMaybeUtf8;
assert_eq!("caf\u{e9}".into_maybe_utf8(), b"caf\xc3\xa9".into_maybe_utf8());
```

*/

#![feature(core)]

use std::{str, char, fmt};
use std::borrow::{IntoCow, Cow, ToOwned};
use std::default::Default;
use std::cmp::Ordering;
use std::iter::{IntoIterator, FromIterator};

/// Byte container optionally encoded as UTF-8. It might be either...
///
/// - Definitely UTF-8-encoded string, or
/// - Bytes. It may be encoded in UTF-8 or other encodings, or it may be simply invalid.
#[derive(Clone)]
pub struct MaybeUtf8Buf { inner: Buf }

// private so that we can tweak the internals when an unsized `MaybeUtf8` can be implemented
#[derive(Clone)]
enum Buf {
    Utf8(String),
    Bytes(Vec<u8>),
}

/// Byte slice optionally encoded as UTF-8. A borrowed version of `MaybeUtf8Buf`.
//
// Rust: this cannot yet be an unsized item. that's why this is not named `MaybeUtf8`. (#16812)
pub struct MaybeUtf8Slice<'a> { inner: Slice<'a> }

enum Slice<'a> {
    Utf8(&'a str),
    Bytes(&'a [u8]),
}

impl MaybeUtf8Buf {
    /// Creates a new empty `MaybeUtf8Buf` value (which is, naturally, encoded in UTF-8).
    pub fn new() -> MaybeUtf8Buf {
        MaybeUtf8Buf { inner: Buf::Utf8(String::new()) }
    }

    /// Creates a `MaybeUtf8Buf` value from an owned `String`.
    pub fn from_str(s: String) -> MaybeUtf8Buf {
        MaybeUtf8Buf { inner: Buf::Utf8(s) }
    }

    /// Creates a `MaybeUtf8Buf` value from an owned `Vec` of `u8` bytes.
    pub fn from_bytes(v: Vec<u8>) -> MaybeUtf8Buf {
        MaybeUtf8Buf { inner: Buf::Bytes(v) }
    }

    // ---8<---
    // the following methods are here due to the inability to
    // implement `Deref` in the current form.

    /// Returns a slice of underlying bytes. It might or might not be encoded in UTF-8.
    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        match self.inner {
            Buf::Utf8(ref s) => s.as_bytes(),
            Buf::Bytes(ref v) => &v,
        }
    }

    /// Returns a string slice encoded in UTF-8 if possible.
    /// It returns `None` if the underlying bytes are not encoded in UTF-8.
    pub fn as_str<'a>(&'a self) -> Option<&'a str> {
        match self.inner {
            Buf::Utf8(ref s) => Some(&s),
            Buf::Bytes(ref v) => str::from_utf8(&v).ok(),
        }
    }

    /// Returns a `Cow` string which represents the current `MaybeUtf8Slice`.
    /// It may call given `to_cow` function to get a `Cow` string out of the bytes.
    /// `to_cow` function itself may return a `String` or `&str` compatible to `Cow` string.
    pub fn map_as_cow<'a, F, T>(&'a self, mut to_cow: F) -> Cow<'a, str>
            where F: FnMut(&'a [u8]) -> T, T: IntoCow<'a, str> {
        match self.inner {
            Buf::Utf8(ref s) => s[..].into_cow(),
            Buf::Bytes(ref v) => to_cow(&v).into_cow(),
        }
    }

    // there is no `as_cow`; if we can convert bytes to a str, we don't need `Cow` string at all.

    /// Returns a `Cow` string which represents the current `MaybeUtf8Slice`.
    /// Any invalid UTF-8 sequences are replaced by U+FFFD, as like `String::from_utf8_lossy`.
    pub fn as_cow_lossy<'a>(&'a self) -> Cow<'a, str> {
        self.map_as_cow(String::from_utf8_lossy)
    }

    // the end of duplicate methods.
    // ---8<---

    /// Returns a `MaybeUtf8Slice` borrowed from this `MaybeUtf8Buf`.
    pub fn to_slice<'a>(&'a self) -> MaybeUtf8Slice<'a> {
        match self.inner {
            Buf::Utf8(ref s) => MaybeUtf8Slice::from_str(s),
            Buf::Bytes(ref v) => MaybeUtf8Slice::from_bytes(v),
        }
    }

    /// Tries to convert a `MaybeUtf8Buf` into a `String`.
    /// If there is an invalid UTF-8 sequence it returns the original `MaybeUtf8Buf` back.
    pub fn into_str(self) -> Result<String, MaybeUtf8Buf> {
        match self.inner {
            Buf::Utf8(s) => Ok(s),
            Buf::Bytes(v) => match String::from_utf8(v) {
                Ok(s) => Ok(s),
                Err(e) => Err(MaybeUtf8Buf { inner: Buf::Bytes(e.into_bytes()) }),
            },
        }
    }

    /// Converts a `MaybeUtf8Buf` into a `String`.
    /// It may call given `into_str` function to get a `String` out of the bytes.
    pub fn map_into_str<F>(self, mut into_str: F) -> String
            where F: FnMut(Vec<u8>) -> String {
        match self.inner {
            Buf::Utf8(s) => s,
            Buf::Bytes(v) => into_str(v),
        }
    }

    /// Converts a `MaybeUtf8Buf` into a `String`.
    /// Any invalid UTF-8 sequences are replaced by U+FFFD, as like `String::from_utf8_lossy`.
    pub fn into_str_lossy(self) -> String {
        self.map_into_str(|v| match String::from_utf8_lossy(v.as_slice()) {
            // `v` is definitely UTF-8, so do not make a copy!
            Cow::Borrowed(_) => unsafe {String::from_utf8_unchecked(v)},
            Cow::Owned(s) => s,
        })
    }

    /// Converts a `MaybeUtf8Buf` into a `Vec` of `u8` bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        match self.inner {
            Buf::Utf8(s) => s.into_bytes(),
            Buf::Bytes(v) => v,
        }
    }

    /// Returns a byte length of the `MaybeUtf8Buf` value.
    pub fn len(&self) -> usize {
        match self.inner {
            Buf::Utf8(ref s) => s.len(),
            Buf::Bytes(ref v) => v.len(),
        }
    }
}

impl<'a> MaybeUtf8Slice<'a> {
    /// Creates a new empty `MaybeUtf8Slice` value (which is, naturally, encoded in UTF-8).
    pub fn new() -> MaybeUtf8Slice<'static> {
        MaybeUtf8Slice { inner: Slice::Utf8("") }
    }

    /// Creates a `MaybeUtf8Slice` reference from a string slice.
    pub fn from_str(s: &'a str) -> MaybeUtf8Slice<'a> {
        MaybeUtf8Slice { inner: Slice::Utf8(s) }
    }

    /// Creates a `MaybeUtf8Slice` reference from a `u8` slice.
    pub fn from_bytes(v: &'a [u8]) -> MaybeUtf8Slice<'a> {
        MaybeUtf8Slice { inner: Slice::Bytes(v) }
    }

    /// Returns a slice of underlying bytes. It might or might not be encoded in UTF-8.
    pub fn as_bytes(&self) -> &'a [u8] {
        match self.inner {
            Slice::Utf8(s) => s.as_bytes(),
            Slice::Bytes(v) => v.as_slice(),
        }
    }

    /// Returns a string slice encoded in UTF-8 if possible.
    /// It returns `None` if the underlying bytes are not encoded in UTF-8.
    pub fn as_str(&self) -> Option<&'a str> {
        match self.inner {
            Slice::Utf8(s) => Some(s.as_slice()),
            Slice::Bytes(v) => str::from_utf8(v.as_slice()).ok(),
        }
    }

    /// Returns a `Cow` string which represents the current `MaybeUtf8Slice`.
    /// It may call given `to_cow` function to get a `Cow` string out of the bytes.
    /// `to_cow` function itself may return a `String` or `&str` compatible to `Cow` string.
    pub fn map_as_cow<F, T>(&self, mut to_cow: F) -> Cow<'a, str>
            where F: FnMut(&'a [u8]) -> T, T: IntoCow<'a, str> {
        match self.inner {
            Slice::Utf8(s) => s.into_cow(),
            Slice::Bytes(v) => to_cow(&v).into_cow(),
        }
    }


    // there is no `as_cow`; if we can convert bytes to a str, we don't need `Cow` string at all.

    /// Returns a `Cow` string which represents the current `MaybeUtf8Slice`.
    /// Any invalid UTF-8 sequences are replaced by U+FFFD, as like `String::from_utf8_lossy`.
    pub fn as_cow_lossy(&self) -> Cow<'a, str> {
        self.map_as_cow(String::from_utf8_lossy)
    }

    /// Returns a new `MaybeUtf8Buf` from the current `MaybeUtf8Slice`.
    pub fn to_owned(&self) -> MaybeUtf8Buf {
        match self.inner {
            Slice::Utf8(s) => MaybeUtf8Buf::from_str(s.to_owned()),
            Slice::Bytes(v) => MaybeUtf8Buf::from_bytes(v.to_owned()),
        }
    }

    /// Returns a byte length of the `MaybeUtf8Slice` value.
    pub fn len(&self) -> usize {
        match self.inner {
            Slice::Utf8(ref s) => s.len(),
            Slice::Bytes(ref v) => v.len(),
        }
    }
}

macro_rules! define_partial_eq_and_cmp {
    ($($lty:ty:$lmeth:ident, $rty:ty:$rmeth:ident;)*) => ($(
        impl<'a, 'b> PartialEq<$rty> for $lty {
            fn eq(&self, other: &$rty) -> bool { self.$lmeth().eq(other.$rmeth()) }
        }
        impl<'a, 'b> PartialOrd<$rty> for $lty {
            fn partial_cmp(&self, other: &$rty) -> Option<Ordering> {
                self.$lmeth().partial_cmp(other.$rmeth())
            }
        }
    )*)
}

define_partial_eq_and_cmp! {
    MaybeUtf8Buf:as_bytes, MaybeUtf8Buf:as_bytes;
    MaybeUtf8Buf:as_bytes, MaybeUtf8Slice<'b>:as_bytes;
    MaybeUtf8Buf:as_bytes, &'b str:as_bytes;
    MaybeUtf8Buf:as_bytes, &'b [u8]:as_slice;
    MaybeUtf8Slice<'a>:as_bytes, MaybeUtf8Buf:as_bytes;
    MaybeUtf8Slice<'a>:as_bytes, MaybeUtf8Slice<'b>:as_bytes;
    MaybeUtf8Slice<'a>:as_bytes, &'b str:as_bytes;
    MaybeUtf8Slice<'a>:as_bytes, &'b [u8]:as_slice;
}

impl Eq for MaybeUtf8Buf {
}

impl<'a> Eq for MaybeUtf8Slice<'a> {
}

impl Ord for MaybeUtf8Buf {
    fn cmp(&self, other: &MaybeUtf8Buf) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl<'a> Ord for MaybeUtf8Slice<'a> {
    fn cmp(&self, other: &MaybeUtf8Slice<'a>) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl FromIterator<char> for MaybeUtf8Buf {
    fn from_iter<I: IntoIterator<Item=char>>(iterator: I) -> MaybeUtf8Buf {
        MaybeUtf8Buf::from_str(FromIterator::from_iter(iterator))
    }
}

impl FromIterator<u8> for MaybeUtf8Buf {
    fn from_iter<I: IntoIterator<Item=u8>>(iterator: I) -> MaybeUtf8Buf {
        MaybeUtf8Buf::from_bytes(FromIterator::from_iter(iterator))
    }
}

impl Default for MaybeUtf8Buf {
    fn default() -> MaybeUtf8Buf { MaybeUtf8Buf::new() }
}

impl<'a> Default for MaybeUtf8Slice<'a> {
    fn default() -> MaybeUtf8Slice<'a> { MaybeUtf8Slice::new() }
}

impl fmt::Debug for MaybeUtf8Buf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_slice(), f)
    }
}

impl fmt::Display for MaybeUtf8Buf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.to_slice(), f)
    }
}

impl<'a> fmt::Debug for MaybeUtf8Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Slice::Utf8(ref s) => fmt::Debug::fmt(s, f),
            Slice::Bytes(ref v) => {
                try!(write!(f, "b\""));
                for &c in v.iter() {
                    match c {
                        b'\t' => try!(write!(f, "\\t")),
                        b'\r' => try!(write!(f, "\\r")),
                        b'\n' => try!(write!(f, "\\n")),
                        b'\\' => try!(write!(f, "\\\\")),
                        b'\'' => try!(write!(f, "\\'")),
                        b'"'  => try!(write!(f, "\\\"")),
                        b'\x20' ... b'\x7e' => try!(write!(f, "{}", c as char)),
                        _ => try!(write!(f, "\\x{}{}",
                                         char::from_digit((c as u32) >> 4, 16).unwrap(),
                                         char::from_digit((c as u32) & 0xf, 16).unwrap()))
                    }
                }
                write!(f, "\"")
            }
        }
    }
}

impl<'a> fmt::Display for MaybeUtf8Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Slice::Utf8(ref s) => fmt::Display::fmt(s, f),
            Slice::Bytes(ref v) => fmt::Display::fmt(&String::from_utf8_lossy(&*v), f),
        }
    }
}

/// A helper trait for uniformly creating `MaybeUtf8Buf` or `MaybeUtf8Slice` values.
pub trait IntoMaybeUtf8<T> {
    /// Converts given value into either `MaybeUtf8Buf` or `MaybeUtf8Slice`.
    fn into_maybe_utf8(self) -> T;
}

impl IntoMaybeUtf8<MaybeUtf8Buf> for String {
    fn into_maybe_utf8(self) -> MaybeUtf8Buf { MaybeUtf8Buf::from_str(self) }
}

impl IntoMaybeUtf8<MaybeUtf8Buf> for Vec<u8> {
    fn into_maybe_utf8(self) -> MaybeUtf8Buf { MaybeUtf8Buf::from_bytes(self) }
}

impl<'a> IntoMaybeUtf8<MaybeUtf8Slice<'a>> for &'a String {
    fn into_maybe_utf8(self) -> MaybeUtf8Slice<'a> { MaybeUtf8Slice::from_str(self) }
}

impl<'a> IntoMaybeUtf8<MaybeUtf8Slice<'a>> for &'a str {
    fn into_maybe_utf8(self) -> MaybeUtf8Slice<'a> { MaybeUtf8Slice::from_str(self) }
}

impl<'a> IntoMaybeUtf8<MaybeUtf8Slice<'a>> for &'a Vec<u8> {
    fn into_maybe_utf8(self) -> MaybeUtf8Slice<'a> { MaybeUtf8Slice::from_bytes(self) }
}

impl<'a> IntoMaybeUtf8<MaybeUtf8Slice<'a>> for &'a [u8] {
    fn into_maybe_utf8(self) -> MaybeUtf8Slice<'a> { MaybeUtf8Slice::from_bytes(self) }
}

