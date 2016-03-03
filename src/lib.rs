// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Terminfo database interface.

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::io::BufReader;
use std::path::Path;

use self::searcher::get_dbpath_for_term;
use self::parser::compiled::parse;


/// A parsed terminfo database entry.
#[derive(Debug, Clone)]
pub struct TermInfo {
    /// Names for the terminal
    pub names: Vec<String>,
    /// Map of capability name to boolean value
    pub bools: HashMap<&'static str, bool>,
    /// Map of capability name to numeric value
    pub numbers: HashMap<&'static str, u16>,
    /// Map of capability name to raw (unexpanded) string
    pub strings: HashMap<&'static str, Vec<u8>>,
}

impl TermInfo {
    /// Create a TermInfo for the named terminal.
    pub fn from_name(name: &str) -> io::Result<TermInfo> {
        get_dbpath_for_term(name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "database not found"))
            .and_then(|p| TermInfo::from_path(&p))
    }

    /// Parse the given TermInfo.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<TermInfo> {
        Self::_from_path(path.as_ref())
    }
    // Keep the metadata small
    // (That is, this uses a &Path so that this function need not be instantiated
    // for every type
    // which implements AsRef<Path>. One day, if/when rustc is a bit smarter, it
    // might do this for
    // us. Alas. )
    fn _from_path(path: &Path) -> io::Result<TermInfo> {
        let file = try!(File::open(path));
        let mut reader = BufReader::new(file);
        parse(&mut reader, false)
    }
}

#[derive(Debug)]
/// An error from parsing a terminfo entry
pub enum Error {
    /// The "magic" number at the start of the file was wrong.
    ///
    /// It should be `0x11A`
    BadMagic(u16),
    /// The names in the file were not valid UTF-8.
    ///
    /// In theory these should only be ASCII, but to work with the Rust `str` type, we treat them
    /// as UTF-8. This is valid, except when a terminfo file decides to be invalid. This hasn't
    /// been encountered in the wild.
    NotUtf8(::std::str::Utf8Error),
    /// The names section of the file was empty
    ShortNames,
    /// More boolean parameters are present in the file than this crate knows how to interpret.
    TooManyBools,
    /// More number parameters are present in the file than this crate knows how to interpret.
    TooManyNumbers,
    /// More string parameters are present in the file than this crate knows how to interpret.
    TooManyStrings,
    /// The length of some field was not >= -1.
    InvalidLength,
    /// The names table was missing a trailing null terminator.
    NamesMissingNull,
    /// The strings table was missing a trailing null terminator.
    StringsMissingNull,
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use std::error::Error;
        use Error::*;
        match *self {
            NotUtf8(e) => write!(f, "{}", e),
            BadMagic(v) => write!(f, "bad magic number {:x} in terminfo header", v),
            _ => f.write_str(self.description()),
        }
    }
}

impl ::std::convert::From<::std::string::FromUtf8Error> for Error {
    fn from(v: ::std::string::FromUtf8Error) -> Self {
        Error::NotUtf8(v.utf8_error())
    }
}

impl ::std::convert::From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;
        match *self {
            BadMagic(..) => "incorrect magic number at start of file",
            ShortNames => "no names exposed, need at least one",
            TooManyBools => "more boolean properties than libterm knows about",
            TooManyNumbers => "more number properties than libterm knows about",
            TooManyStrings => "more string properties than libterm knows about",
            InvalidLength => "invalid length field value, must be >= -1",
            NotUtf8(ref e) => e.description(),
            NamesMissingNull => "names table missing NUL terminator",
            StringsMissingNull => "string table missing NUL terminator",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        use Error::*;
        match *self {
            NotUtf8(ref e) => Some(e),
            _ => None,
        }
    }
}

pub mod searcher;

/// TermInfo format parsing.
pub mod parser {
    //! ncurses-compatible compiled terminfo format parsing (term(5))
    pub mod compiled;
    mod names;
}
pub mod parm;
