#[cfg(all(not(target_arch = "wasm32"), test))]
mod test;

use liblumen_alloc::erts::exception;
use liblumen_alloc::erts::process::Process;
use liblumen_alloc::erts::term::prelude::*;
use crate::runtime::context::term_is_not_integer;
use anyhow::*;

/// Returns the smallest possible representation in a binary digit representation for the given big endian unsigned integer.
#[native_implemented::function(erlang:encode_unsigned/1)]
pub fn result(process: &Process, term: Term) -> exception::Result<Term> {
    match term.decode().unwrap() {
        TypedTerm::SmallInteger(small_integer) => {
            let mut bytes: Vec<u8> = small_integer
                .to_le_bytes()
                .iter()
                .filter_map(|&b| match b {
                    0 => None,
                    b => Some(b)
                })
                .collect();

            bytes.reverse();

            Ok(process.binary_from_bytes(&bytes))
        },
        TypedTerm::BigInteger(big_integer) => {
            let bytes: Vec<u8> = big_integer
                .to_signed_bytes_be()
                .iter()
                .filter_map(|&b| match b {
                    0 => None,
                    b => Some(b)
                })
                .collect();

            Ok(process.binary_from_bytes(&bytes))
        },
        _ => Err(TypeError)
            .context(term_is_not_integer(
                "encoded_unsigned",
                term
            ))
            .map_err(From::from),
    }
}
