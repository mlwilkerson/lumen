#[path = "spawn_opt_4/with_atom_module.rs"]
mod with_atom_module;

test_stdout!(
    without_atom_module_errors_badarg,
    "{caught, error, badarg}\n"
);
