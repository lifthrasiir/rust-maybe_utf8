// maybe_utf8: Byte container optionally encoded as UTF-8.
// Copyright (c) 2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!

# MaybeUTF8 0.1.1

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
extern crate maybe_utf8;

use std::borrow::IntoCow;
use encoding::{Encoding, DecoderTrap};
use encoding::all::ISO_8859_2;
use maybe_utf8::MaybeUTF8;

fn main() {
    let namebuf = MaybeUTF8::from_bytes(vec![99,97,102,233]);
    let name = namebuf.map_into_str(|v| ISO_8859_2.decode(&*v, DecoderTrap::Replace).unwrap());
    assert_eq!(name, "caf\u{e9}");
}
```

*/

#![allow(unstable)]

use std::{str, char, fmt};
use std::borrow::{IntoCow, Cow};
use std::string::CowString;
use std::default::Default;
use std::path::BytesContainer;
use std::cmp::Ordering;
use std::iter::FromIterator;

/// Byte container optionally encoded as UTF-8.
#[derive(Clone)]
pub enum MaybeUTF8 {
    /// Definitely UTF-8-encoded string.
    UTF8(String),
    /// Bytes. It may be encoded in UTF-8 or other encodings, or it may be simply invalid.
    Bytes(Vec<u8>),
}

impl MaybeUTF8 {
    /// Creates a new empty `MaybeUTF8` value (which is, naturally, encoded in UTF-8).
    pub fn new() -> MaybeUTF8 {
        MaybeUTF8::UTF8(String::new())
    }

    /// Creates a `MaybeUTF8` value from an owned `String`.
    pub fn from_str(s: String) -> MaybeUTF8 {
        MaybeUTF8::UTF8(s)
    }

    /// Creates a `MaybeUTF8` value from an owned `Vec` of `u8` bytes.
    pub fn from_bytes(v: Vec<u8>) -> MaybeUTF8 {
        MaybeUTF8::Bytes(v)
    }

    /// Returns a slice of underlying bytes. It might or might not be encoded in UTF-8.
    pub fn as_bytes(&self) -> &[u8] {
        match *self {
            MaybeUTF8::UTF8(ref s) => s.as_bytes(),
            MaybeUTF8::Bytes(ref v) => v.as_slice(),
        }
    }

    /// Returns a string slice encoded in UTF-8 if possible.
    /// It returns `None` if the underlying bytes are not encoded in UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            MaybeUTF8::UTF8(ref s) => Some(s.as_slice()),
            MaybeUTF8::Bytes(ref v) => str::from_utf8(v.as_slice()).ok(),
        }
    }

    /// Returns a `CowString` which represents the current `MaybeUTF8`.
    /// It may call given `as_cow` function to get a `CowString` out of the bytes.
    pub fn map_as_cow<'a, F>(&'a self, mut as_cow: F) -> CowString<'a>
            where F: FnMut(&'a [u8]) -> CowString<'a> {
        match *self {
            MaybeUTF8::UTF8(ref s) => s.as_slice().into_cow(),
            MaybeUTF8::Bytes(ref v) => as_cow(v.as_slice()),
        }
    }

    // there is no `as_cow`; if we can convert bytes to a str, we don't need `CowString` at all.

    /// Returns a `CowString` which represents the current `MaybeUTF8`.
    /// Any invalid UTF-8 sequences are replaced by U+FFFD, as like `String::from_utf8_lossy`.
    pub fn as_cow_lossy<'a>(&'a self) -> CowString<'a> {
        self.map_as_cow(String::from_utf8_lossy)
    }

    /// Tries to convert a `MaybeUTF8` into a `String`.
    /// If there is an invalid UTF-8 sequence it returns the original `MaybeUTF8` back.
    pub fn into_str(self) -> Result<String, MaybeUTF8> {
        match self {
            MaybeUTF8::UTF8(s) => Ok(s),
            MaybeUTF8::Bytes(v) => match String::from_utf8(v) {
                Ok(s) => Ok(s),
                Err(e) => Err(MaybeUTF8::Bytes(e.into_bytes())),
            },
        }
    }

    /// Converts a `MaybeUTF8` into a `String`.
    /// It may call given `into_str` function to get a `String` out of the bytes.
    pub fn map_into_str<F>(self, mut into_str: F) -> String
            where F: FnMut(Vec<u8>) -> String {
        match self {
            MaybeUTF8::UTF8(s) => s,
            MaybeUTF8::Bytes(v) => into_str(v),
        }
    }

    /// Converts a `MaybeUTF8` into a `String`.
    /// Any invalid UTF-8 sequences are replaced by U+FFFD, as like `String::from_utf8_lossy`.
    pub fn into_str_lossy(self) -> String {
        self.map_into_str(|v| match String::from_utf8_lossy(v.as_slice()) {
            // `v` is definitely UTF-8, so do not make a copy!
            Cow::Borrowed(_) => unsafe {String::from_utf8_unchecked(v)},
            Cow::Owned(s) => s,
        })
    }

    /// Converts a `MaybeUTF8` into a `Vec` of `u8` bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            MaybeUTF8::UTF8(s) => s.into_bytes(),
            MaybeUTF8::Bytes(v) => v,
        }
    }

    /// Returns a byte length of the `MaybeUTF8` value.
    pub fn len(&self) -> usize {
        match *self {
            MaybeUTF8::UTF8(ref s) => s.len(),
            MaybeUTF8::Bytes(ref v) => v.len(),
        }
    }
}

macro_rules! define_partial_eq_and_cmp {
    ($($lty:ty:$lmeth:ident, $rty:ty:$rmeth:ident;)*) => ($(
        impl<'a> PartialEq<$rty> for $lty {
            fn eq(&self, other: &$rty) -> bool { self.$lmeth().eq(other.$rmeth()) }
        }
        impl<'a> PartialOrd<$rty> for $lty {
            fn partial_cmp(&self, other: &$rty) -> Option<Ordering> {
                self.$lmeth().partial_cmp(other.$rmeth())
            }
        }
    )*)
}

define_partial_eq_and_cmp! {
    MaybeUTF8:as_bytes, MaybeUTF8:as_bytes;
    MaybeUTF8:as_bytes, &'a str:as_bytes;
    MaybeUTF8:as_bytes, &'a [u8]:as_slice;
}

impl Eq for MaybeUTF8 {
}

impl Ord for MaybeUTF8 {
    fn cmp(&self, other: &MaybeUTF8) -> Ordering {
        self.as_bytes().cmp(other.container_as_bytes())
    }
}

impl BytesContainer for MaybeUTF8 {
    fn container_as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn container_as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn is_str(_: Option<&MaybeUTF8>) -> bool {
        false
    }
}

impl FromIterator<char> for MaybeUTF8 {
    fn from_iter<I: Iterator<Item=char>>(iterator: I) -> MaybeUTF8 {
        MaybeUTF8::from_str(FromIterator::from_iter(iterator))
    }
}

impl FromIterator<u8> for MaybeUTF8 {
    fn from_iter<I: Iterator<Item=u8>>(iterator: I) -> MaybeUTF8 {
        MaybeUTF8::from_bytes(FromIterator::from_iter(iterator))
    }
}

impl Default for MaybeUTF8 {
    fn default() -> MaybeUTF8 {
        MaybeUTF8::new()
    }
}

impl fmt::Debug for MaybeUTF8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MaybeUTF8::UTF8(ref s) => fmt::Debug::fmt(s, f),
            MaybeUTF8::Bytes(ref v) => {
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
                                         char::from_digit((c as usize) >> 4, 16).unwrap(),
                                         char::from_digit((c as usize) & 0xf, 16).unwrap()))
                    }
                }
                write!(f, "\"")
            }
        }
    }
}

impl fmt::Display for MaybeUTF8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MaybeUTF8::UTF8(ref s) => fmt::Display::fmt(s, f),
            MaybeUTF8::Bytes(ref v) => fmt::Display::fmt(&String::from_utf8_lossy(&**v), f),
        }
    }
}

