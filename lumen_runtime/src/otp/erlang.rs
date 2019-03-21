//! Mirrors [erlang](http://erlang::org/doc/man/erlang::html) module

#![cfg_attr(not(test), allow(dead_code))]

use std::cmp::Ordering;
use std::num::FpCategory;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::atom::Existence;
use crate::binary::{heap, sub, Part, ToTerm, ToTermOptions};
use crate::exception::Result;
use crate::float::Float;
use crate::integer::{big, small};
use crate::list::Cons;
use crate::map::Map;
use crate::otp;
use crate::process::{IntoProcess, Process, TryIntoInProcess};
use crate::term::{Tag, Term};
use crate::time;
use crate::tuple::Tuple;

#[cfg(test)]
mod tests;

pub fn abs(number: Term, mut process: &mut Process) -> Result {
    match number.tag() {
        Tag::SmallInteger => {
            if unsafe { number.small_integer_is_negative() } {
                // cast first so that sign bit is extended on shift
                let signed = (number.tagged as isize) >> Tag::SMALL_INTEGER_BIT_COUNT;
                let positive = -signed;
                Ok(Term {
                    tagged: ((positive << Tag::SMALL_INTEGER_BIT_COUNT) as usize)
                        | (Tag::SmallInteger as usize),
                })
            } else {
                Ok(Term {
                    tagged: number.tagged,
                })
            }
        }
        Tag::Boxed => {
            let unboxed: &Term = number.unbox_reference();

            match unboxed.tag() {
                Tag::BigInteger => {
                    let big_integer: &big::Integer = number.unbox_reference();
                    let big_int = &big_integer.inner;
                    let zero_big_int: &BigInt = &Zero::zero();

                    let positive_term: Term = if big_int < zero_big_int {
                        let positive_big_int: BigInt = -1 * big_int;

                        positive_big_int.into_process(&mut process)
                    } else {
                        number
                    };

                    Ok(positive_term)
                }
                Tag::Float => {
                    let float: &Float = number.unbox_reference();
                    let inner = float.inner;

                    match inner.partial_cmp(&0.0).unwrap() {
                        Ordering::Less => {
                            let positive_inner = inner.abs();
                            let positive_number: Term = positive_inner.into_process(&mut process);

                            Ok(positive_number)
                        }
                        _ => Ok(number),
                    }
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

pub fn append_element(tuple: Term, element: Term, mut process: &mut Process) -> Result {
    let internal: &Tuple = tuple.try_into_in_process(&mut process)?;
    let new_tuple = internal.append_element(element, &mut process.term_arena);

    Ok(new_tuple.into())
}

pub fn atom_to_binary(atom: Term, encoding: Term, mut process: &mut Process) -> Result {
    if let Tag::Atom = atom.tag() {
        encoding.atom_to_encoding(&mut process)?;
        let string = atom.atom_to_string(process);
        Ok(Term::slice_to_binary(string.as_bytes(), &mut process))
    } else {
        Err(bad_argument!(&mut process))
    }
}

pub fn atom_to_list(atom: Term, encoding: Term, mut process: &mut Process) -> Result {
    if let Tag::Atom = atom.tag() {
        encoding.atom_to_encoding(&mut process)?;
        let string = atom.atom_to_string(process);
        Ok(Term::chars_to_list(string.chars(), &mut process))
    } else {
        Err(bad_argument!(&mut process))
    }
}

pub fn binary_part(binary: Term, start: Term, length: Term, mut process: &mut Process) -> Result {
    match binary.tag() {
        Tag::Boxed => {
            let unboxed: &Term = binary.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = binary.unbox_reference();

                    heap_binary.part(start, length, &mut process)
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = binary.unbox_reference();

                    subbinary.part(start, length, &mut process)
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

pub fn binary_to_atom(binary: Term, encoding: Term, process: &mut Process) -> Result {
    binary_existence_to_atom(binary, encoding, Existence::DoNotCare, process)
}

pub fn binary_to_existing_atom(binary: Term, encoding: Term, process: &mut Process) -> Result {
    binary_existence_to_atom(binary, encoding, Existence::Exists, process)
}

pub fn binary_to_float(binary: Term, mut process: &mut Process) -> Result {
    let string: String = binary.try_into_in_process(&mut process)?;

    match string.parse::<f64>() {
        Ok(inner) => {
            match inner.classify() {
                FpCategory::Normal | FpCategory::Subnormal =>
                // unlike Rust, Erlang requires float strings to have a decimal point
                {
                    if (inner.fract() == 0.0) & !string.chars().any(|b| b == '.') {
                        Err(bad_argument!(&mut process))
                    } else {
                        Ok(inner.into_process(&mut process))
                    }
                }
                // Erlang has no support for Nan, +inf or -inf
                FpCategory::Nan | FpCategory::Infinite => Err(bad_argument!(&mut process)),
                FpCategory::Zero => {
                    // Erlang does not track the difference without +0 and -0.
                    Ok(inner.abs().into_process(&mut process))
                }
            }
        }
        Err(_) => Err(bad_argument!(&mut process)),
    }
}

/// `binary_to_integer/1`
pub fn binary_to_integer(binary: Term, mut process: &mut Process) -> Result {
    let string: String = binary.try_into_in_process(&mut process)?;
    let bytes = string.as_bytes();

    match BigInt::parse_bytes(bytes, 10) {
        Some(big_int) => {
            let term: Term = big_int.into_process(&mut process);

            Ok(term)
        }
        None => Err(bad_argument!(&mut process)),
    }
}

/// `binary_to_integer/2`
pub fn binary_in_base_to_integer(binary: Term, base: Term, mut process: &mut Process) -> Result {
    let string: String = binary.try_into_in_process(&mut process)?;
    let radix: usize = base.try_into_in_process(&mut process)?;

    if 2 <= radix && radix <= 36 {
        let bytes = string.as_bytes();

        match BigInt::parse_bytes(bytes, radix as u32) {
            Some(big_int) => {
                let term: Term = big_int.into_process(&mut process);

                Ok(term)
            }
            None => Err(bad_argument!(&mut process)),
        }
    } else {
        Err(bad_argument!(&mut process))
    }
}

/// `binary_to_list/1`
pub fn binary_to_list(binary: Term, mut process: &mut Process) -> Result {
    match binary.tag() {
        Tag::Boxed => {
            let unboxed: &Term = binary.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = binary.unbox_reference();

                    Ok(heap_binary.to_list(&mut process))
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = binary.unbox_reference();

                    subbinary.to_list(&mut process)
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

/// `binary_to_list/3`
///
/// The one-based indexing for binaries used by this function is deprecated. New code is to use
/// [crate::otp::binary::bin_to_list] instead. All functions in module [crate::otp::binary]
/// consistently use zero-based indexing.
pub fn binary_byte_range_to_list(
    binary: Term,
    start: Term,
    stop: Term,
    mut process: &mut Process,
) -> Result {
    let one_based_start_usize: usize = start.try_into_in_process(&mut process)?;

    if 1 <= one_based_start_usize {
        let one_based_stop_usize: usize = stop.try_into_in_process(&mut process)?;

        if one_based_start_usize <= one_based_stop_usize {
            let zero_based_start_usize = one_based_start_usize - 1;
            let zero_based_stop_usize = one_based_stop_usize - 1;

            let length_usize = zero_based_stop_usize - zero_based_stop_usize + 1;

            otp::binary::bin_to_list(
                binary,
                zero_based_start_usize.into_process(&mut process),
                length_usize.into_process(&mut process),
                &mut process,
            )
        } else {
            Err(bad_argument!(&mut process))
        }
    } else {
        Err(bad_argument!(&mut process))
    }
}

/// `binary_to_term/1`
pub fn binary_to_term(binary: Term, process: &mut Process) -> Result {
    binary_options_to_term(binary, Term::EMPTY_LIST, process)
}

/// `binary_to_term/2`
pub fn binary_options_to_term(binary: Term, options: Term, mut process: &mut Process) -> Result {
    let to_term_options: ToTermOptions = options.try_into_in_process(process)?;

    match binary.tag() {
        Tag::Boxed => {
            let unboxed: &Term = binary.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = binary.unbox_reference();

                    heap_binary.to_term(to_term_options, &mut process)
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = binary.unbox_reference();

                    subbinary.to_term(to_term_options, &mut process)
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

pub fn bit_size(bit_string: Term, mut process: &mut Process) -> Result {
    match bit_string.tag() {
        Tag::Boxed => {
            let unboxed: &Term = bit_string.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = bit_string.unbox_reference();

                    Ok(heap_binary.bit_size())
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = bit_string.unbox_reference();

                    Ok(subbinary.bit_size())
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
    .map(|bit_size_usize| bit_size_usize.into_process(&mut process))
}

pub fn bitstring_to_list(bit_string: Term, mut process: &mut Process) -> Result {
    match bit_string.tag() {
        Tag::Boxed => {
            let unboxed: &Term = bit_string.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = bit_string.unbox_reference();

                    Ok(heap_binary.to_bitstring_list(&mut process))
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = bit_string.unbox_reference();

                    Ok(subbinary.to_bitstring_list(&mut process))
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

pub fn byte_size(bit_string: Term, mut process: &mut Process) -> Result {
    match bit_string.tag() {
        Tag::Boxed => {
            let unboxed: &Term = bit_string.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = bit_string.unbox_reference();

                    Ok(heap_binary.byte_size())
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = bit_string.unbox_reference();

                    Ok(subbinary.byte_size())
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
    .map(|byte_size_usize| byte_size_usize.into_process(&mut process))
}

pub fn ceil(number: Term, mut process: &mut Process) -> Result {
    match number.tag() {
        Tag::SmallInteger => Ok(number),
        Tag::Boxed => {
            let unboxed: &Term = number.unbox_reference();

            match unboxed.tag() {
                Tag::BigInteger => Ok(number),
                Tag::Float => {
                    let float: &Float = number.unbox_reference();
                    let inner = float.inner;
                    let ceil_inner = inner.ceil();

                    // skip creating a rug::Integer if float can fit in small integer.
                    let ceil_term =
                        if (small::MIN as f64) <= ceil_inner && ceil_inner <= (small::MAX as f64) {
                            (ceil_inner as usize).into_process(&mut process)
                        } else {
                            let ceil_string = ceil_inner.to_string();
                            let ceil_bytes = ceil_string.as_bytes();
                            let big_int = BigInt::parse_bytes(ceil_bytes, 10).unwrap();

                            big_int.into_process(&mut process)
                        };

                    Ok(ceil_term)
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
}

pub fn convert_time_unit(
    time: Term,
    from_unit: Term,
    to_unit: Term,
    mut process: &mut Process,
) -> Result {
    let time_big_int: BigInt = time.try_into_in_process(&mut process)?;
    let from_unit_unit: crate::time::Unit = from_unit.try_into_in_process(&mut process)?;
    let to_unit_unit: crate::time::Unit = to_unit.try_into_in_process(&mut process)?;
    let converted =
        time::convert(time_big_int, from_unit_unit, to_unit_unit).into_process(&mut process);

    Ok(converted)
}

pub fn delete_element(tuple: Term, index: Term, mut process: &mut Process) -> Result {
    let initial_inner_tuple: &Tuple = tuple.try_into_in_process(&mut process)?;
    let inner_index: usize = index.try_into_in_process(&mut process)?;

    initial_inner_tuple
        .delete_element(inner_index, &mut process)
        .map(|final_inner_tuple| final_inner_tuple.into())
}

pub fn element(tuple: Term, index: Term, mut process: &mut Process) -> Result {
    let inner_tuple: &Tuple = tuple.try_into_in_process(&mut process)?;
    let inner_index: usize = index.try_into_in_process(&mut process)?;

    inner_tuple.element(inner_index, &mut process)
}

/// `error/1`
pub fn error(reason: Term) -> Result {
    Err(error!(reason))
}

/// `error/2`
pub fn error_with_arguments(reason: Term, arguments: Term) -> Result {
    Err(error!(reason, arguments))
}

pub fn head(list: Term, mut process: &mut Process) -> Result {
    let cons: &Cons = list.try_into_in_process(&mut process)?;

    Ok(cons.head())
}

pub fn insert_element(
    tuple: Term,
    index: Term,
    element: Term,
    mut process: &mut Process,
) -> Result {
    let initial_inner_tuple: &Tuple = tuple.try_into_in_process(&mut process)?;
    let inner_index: usize = index.try_into_in_process(&mut process)?;

    initial_inner_tuple
        .insert_element(inner_index, element, &mut process)
        .map(|final_inner_tuple| final_inner_tuple.into())
}

pub fn is_atom(term: Term, mut process: &mut Process) -> Term {
    (term.tag() == Tag::Atom).into_process(&mut process)
}

pub fn is_binary(term: Term, mut process: &mut Process) -> Term {
    match term.tag() {
        Tag::Boxed => {
            let unboxed: &Term = term.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => true,
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = term.unbox_reference();

                    subbinary.is_binary()
                }
                _ => false,
            }
        }
        _ => false,
    }
    .into_process(&mut process)
}

pub fn is_integer(term: Term, mut process: &mut Process) -> Term {
    match term.tag() {
        Tag::SmallInteger => true,
        Tag::Boxed => {
            let unboxed: &Term = term.unbox_reference();

            unboxed.tag() == Tag::BigInteger
        }
        _ => false,
    }
    .into_process(&mut process)
}

pub fn is_list(term: Term, mut process: &mut Process) -> Term {
    match term.tag() {
        Tag::EmptyList | Tag::List => true,
        _ => false,
    }
    .into_process(&mut process)
}

pub fn is_map(term: Term, mut process: &mut Process) -> Term {
    match term.tag() {
        Tag::Boxed => {
            let unboxed: &Term = term.unbox_reference();

            match unboxed.tag() {
                Tag::Map => true,
                _ => false,
            }
        }
        _ => false,
    }
    .into_process(&mut process)
}

pub fn is_map_key(key: Term, map: Term, mut process: &mut Process) -> Result {
    match map.tag() {
        Tag::Boxed => {
            let unboxed_map: &Term = map.unbox_reference();

            match unboxed_map.tag() {
                Tag::Map => {
                    let map_map: &Map = map.unbox_reference();
                    Some(map_map.is_key(key).into_process(&mut process))
                }
                _ => None,
            }
        }
        _ => None,
    }
    .ok_or_else(|| {
        let badmap = Term::str_to_atom("badmap", Existence::DoNotCare, &mut process).unwrap();
        let reason = Term::slice_to_tuple(&[badmap, map], &mut process);

        error!(reason)
    })
}

pub fn is_pid(term: Term, mut process: &mut Process) -> Term {
    match term.tag() {
        Tag::LocalPid => true,
        Tag::Boxed => {
            let unboxed: &Term = term.unbox_reference();

            match unboxed.tag() {
                Tag::ExternalPid => true,
                _ => false,
            }
        }
        _ => false,
    }
    .into_process(&mut process)
}

pub fn is_tuple(term: Term, mut process: &mut Process) -> Term {
    (term.tag() == Tag::Boxed && term.unbox_reference::<Term>().tag() == Tag::Arity)
        .into_process(&mut process)
}

pub fn length(list: Term, mut process: &mut Process) -> Result {
    let mut length: usize = 0;
    let mut tail = list;

    loop {
        match tail.tag() {
            Tag::EmptyList => break Ok(length.into_process(&mut process)),
            Tag::List => {
                tail = crate::otp::erlang::tail(tail, &mut process).unwrap();
                length += 1;
            }
            _ => break Err(bad_argument!(&mut process)),
        }
    }
}

pub fn list_to_pid(string: Term, mut process: &mut Process) -> Result {
    let cons: &Cons = string.try_into_in_process(&mut process)?;

    cons.to_pid(&mut process)
}

pub fn self_pid(process: &Process) -> Term {
    process.pid
}

pub fn size(binary_or_tuple: Term, mut process: &mut Process) -> Result {
    match binary_or_tuple.tag() {
        Tag::Boxed => {
            let unboxed: &Term = binary_or_tuple.unbox_reference();

            match unboxed.tag() {
                Tag::Arity => {
                    let tuple: &Tuple = binary_or_tuple.unbox_reference();

                    Ok(tuple.size())
                }
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = binary_or_tuple.unbox_reference();

                    Ok(heap_binary.size())
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = binary_or_tuple.unbox_reference();

                    Ok(subbinary.size())
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
    .map(|integer| integer.into_process(&mut process))
}

pub fn tail(list: Term, process: &mut Process) -> Result {
    let cons: &Cons = list.try_into_in_process(process)?;

    Ok(cons.tail())
}

// Private Functions

fn binary_existence_to_atom(
    binary: Term,
    encoding: Term,
    existence: Existence,
    mut process: &mut Process,
) -> Result {
    encoding.atom_to_encoding(&mut process)?;

    match binary.tag() {
        Tag::Boxed => {
            let unboxed: &Term = binary.unbox_reference();

            match unboxed.tag() {
                Tag::HeapBinary => {
                    let heap_binary: &heap::Binary = binary.unbox_reference();

                    heap_binary.to_atom_index(existence, &mut process)
                }
                Tag::Subbinary => {
                    let subbinary: &sub::Binary = binary.unbox_reference();

                    subbinary.to_atom_index(existence, &mut process)
                }
                _ => Err(bad_argument!(&mut process)),
            }
        }
        _ => Err(bad_argument!(&mut process)),
    }
    .map(|atom_index| atom_index.into())
}
