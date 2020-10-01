use liblumen_alloc::erts::term::prelude::*;

#[native_implemented::function(erlang:is_float/1)]
pub fn result(term: Term) -> Term {
    term.is_boxed_float().into()
}
