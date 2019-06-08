use std::sync::Arc;

use proptest::prop_oneof;
use proptest::strategy::Strategy;

use crate::process::{IntoProcess, Process};
use crate::term::Term;

pub mod big;
pub mod small;

pub fn big(arc_process: Arc<Process>) -> impl Strategy<Value = Term> {
    prop_oneof![
        big::negative(arc_process.clone()),
        big::positive(arc_process)
    ]
}

pub fn non_negative(arc_process: Arc<Process>) -> impl Strategy<Value = Term> {
    prop_oneof![
        small::non_negative(arc_process.clone()),
        big::positive(arc_process)
    ]
}

pub fn small(arc_process: Arc<Process>) -> impl Strategy<Value = Term> {
    (crate::integer::small::MIN..=crate::integer::small::MAX)
        .prop_map(move |i| i.into_process(&arc_process))
}
