use super::*;

use proptest::arbitrary::any;
use proptest::strategy::{BoxedStrategy, Just, Strategy};
use proptest::test_runner::{Config, TestRunner};
use proptest::{prop_assert, prop_assert_eq};

use liblumen_alloc::erts::term::{make_pid, next_pid, HeapBin, SmallInteger, SubBinary};

use crate::otp::erlang;
use crate::process;
use crate::scheduler::{with_process, with_process_arc};
use crate::test::{
    count_ones, has_heap_message, has_message, has_process_message, receive_message,
    registered_name, strategy,
};

mod binary_to_list_3;
mod binary_to_term_1;
mod binary_to_term_2;
mod bit_size_1;
mod bitstring_to_list_1;
mod bnot_1;
mod bor_2;
mod bsl_2;
mod bsr_2;
mod bxor_2;
mod byte_size_1;
mod cancel_timer_1;
mod cancel_timer_2;
mod ceil_1;
mod concatenate_2;
mod delete_element_2;
mod div_2;
mod divide_2;
mod element_2;
mod error_1;
mod error_2;
mod hd_1;
mod insert_element_3;
mod is_alive_0;
mod is_atom_1;
mod is_binary_1;
mod is_bitstring_1;
mod is_boolean_1;
mod is_equal_or_less_than_2;
mod is_float_1;
mod is_greater_than_2;
mod is_greater_than_or_equal_2;
mod is_integer_1;
mod is_less_than_2;
mod is_list_1;
mod is_map_1;
mod is_number_1;
mod is_pid_1;
mod is_record_2;
mod is_record_3;
mod is_reference_1;
mod is_tuple_1;
mod length_1;
mod list_to_atom_1;
mod list_to_binary_1;
mod list_to_bitstring_1;
mod list_to_existing_atom_1;
mod list_to_pid_1;
mod list_to_tuple_1;
mod make_ref_0;
mod map_get_2;
mod map_size_1;
mod max_2;
mod min_2;
mod monotonic_time_1;
mod multiply_2;
mod negate_1;
mod node_0;
mod not_1;
mod or_2;
mod orelse_2;
mod raise_3;
mod read_timer_1;
mod read_timer_2;
mod register_2;
mod registered_0;
mod rem_2;
mod send_3;
mod send_after_3;
mod send_after_4;
mod setelement_3;
mod size_1;
mod split_binary_2;
mod start_timer_3;
mod start_timer_4;
mod subtract_list_2;
mod throw_1;
mod tl_1;
mod tuple_size_1;
mod tuple_to_list_1;
mod unregister_1;
mod whereis_1;
mod xor_2;

enum FirstSecond {
    First,
    Second,
}

fn cancel_timer_message(timer_reference: Term, result: Term, process: &Process) -> Term {
    timer_message("cancel_timer", timer_reference, result, process)
}

fn errors_badarg<F>(actual: F)
where
    F: FnOnce(&Process) -> Result,
{
    with_process(|process| assert_badarg!(actual(&process)))
}

fn errors_badarith<F>(actual: F)
where
    F: FnOnce(&Process) -> Result,
{
    with_process(|process| assert_badarith!(actual(&process)))
}

fn list_term(process: &Process) -> Term {
    let head_term = atom_unchecked("head");

    process.cons(head_term, Term::NIL).unwrap()
}

fn read_timer_message(timer_reference: Term, result: Term, process: &Process) -> Term {
    timer_message("read_timer", timer_reference, result, process)
}

fn timeout_message(timer_reference: Term, message: Term, process: &Process) -> Term {
    timer_message("timeout", timer_reference, message, process)
}

fn timer_message(tag: &str, timer_reference: Term, message: Term, process: &Process) -> Term {
    process
        .tuple_from_slice(&[atom_unchecked(tag), timer_reference, message])
        .unwrap()
}
