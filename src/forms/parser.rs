use chomp::{Input, U8Result};
use chomp::parsers::{take_remainder, take_while, take_while1, token};
use chomp::combinators::{many, or};

use std::str;
use std::u8;

/// The type used for parsing form data
#[derive(Debug)]
pub struct Form<'a> {
    /// The `name` field in an html input
    pub name: &'a [u8],
    /// The value associated with the input field
    pub value: &'a [u8],
}

/// Parse through all form data and return `Vec` of all `Form`s
pub fn form(i: Input<u8>) -> U8Result<Vec<Form>> {
    many(i, form_parser)
}

fn form_parser(i: Input<u8>) -> U8Result<Form> {
    // Suppose that we have a form with two text inputs named
    // `q` and `r`. We also have a button to submit this form
    // with type `submit` and name `action`. We will receive this
    // as `q=&r=&action=`. With appropriate values after the `=`.
    //
    // We need two parsers in order to properly parse this. The first
    // parser handles the data of the form `q=val&`, returning
    // `Form{name = q, value = val}`.
    // The second parser handles data of the form `q=val`, returning
    // the same as the former.
    or(i,
       parser!{
            let name = take_while1(|c| c != b'=');
            token(b'=');
            let value = take_while(|c| c != b'&');
            token(b'&');

            ret Form{
                name: name,
                value: value,
            }
        },
       parser!{
            let name = take_while1(|c| c != b'=');
            token(b'=');
            let value = take_remainder();

            ret Form{
                name: name,
                value: value,
            }
        })
}

/// When we receive form data with enctype
/// `application/x-www-form-urlencoded` any characters
/// that are not `[0-9A-Za-z]`, `*`, `-`, or `_` are
/// replaced by `%XX` where `XX` is the 2 digit hex value
/// of the character. Spaces (' ') are replaced by `+`.
///
/// `replace_special_characters` goes through `data` and
/// replaces all escaped characters with the appropriate
/// character.
///
/// For now we ignore carriage returns, as *nix dislikes them.
///
/// Returns `Err(String)` if we receive invalid input, i.e. if
/// `data` ends with `%` or `%X`.
///
pub fn replace_special_characters(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = vec![];

    let mut data = data.into_iter();

    while let Some(&c) = data.next() {
        if b'%' == c {
            let mut d: Vec<u8> = vec![];

            if let Some(&c) = data.next() {
                d.push(c);
            } else {
                return Err("Unexpected end of input!".to_string());
            }
            if let Some(&c) = data.next() {
                d.push(c);
            } else {
                return Err("Unexpected end of input!".to_string());
            }

            let val = match str::from_utf8(&d[..]) {
                Err(e) => return Err(format!("{}", e)),
                Ok(v) => {
                    match u8::from_str_radix(v, 16) {
                        Ok(v) => v,
                        Err(_) => return Err(format!("Error parsing hex value {}", v)),
                    }
                }
            };

            // For now we are not pushing carriage returns, eventually we
            // should maybe check if we are on Windows or *nix?
            // TODO(nokaa): Handle this properly
            if b'\r' != val {
                buf.push(val);
            }
        } else if b'+' == c {
            buf.push(b' ');
        } else {
            buf.push(c);
        }
    }

    Ok(buf)
}
