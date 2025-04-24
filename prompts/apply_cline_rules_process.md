User intention: Apply the process described in the .clinerules to the project.
This involves:

1. Reading specifications.
2. Checking/writing tests if code changes are implied (not directly applicable
   to this meta-prompt, but relevant for subsequent steps).
3. Applying code documentation rules if code is changed.
4. Performing the prompt (applying the rules).
5. Updating specifications if changes occur.
6. Informing the user of changes.
7. Running unit tests.
8. Following JJ workflow for commits.
9. Ensuring configuration files (rustfmt.toml, Cargo.toml, rust-toolchain.toml,
   .cargo/config.toml) match the standards defined in the rules.
