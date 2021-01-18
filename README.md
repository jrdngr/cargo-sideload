`cargo` supports alternative registries but does not support authenticated downloads.
`cargo-sideload` lets your specify a registry, then reads your `Cargo.lock` file to determine
which dependencies use the specified registry. It then downloads the `.crate` files for those 
dependencies using a user-specified authentication method, and stores them in your local `cargo` 
cache as if they were downloaded directly with `cargo`.

The next time you run a `cargo` command, it will find the cached `.crate` files and use those 
instead of attempting to download them.

`cargo-sideload` is a workaround until `cargo` supports authenticated downloads natively. 

## Installation
`cargo install cargo-sideload`

## First run
1. [Add](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry) your alternate registry to `~/.cargo/config.toml`.
2. [Add](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry) `registry = "[registry-name]"` to any dependencies that use the registry.
3. Run `cargo update` to populate your `Cargo.lock` file.
4. Run `cargo sideload --registry=[registry-name]` in your crate's root.
   - Set `CARGO_SIDELOAD_AUTH_HEADER` in your shell or use the `--auth-header` 
   flag if your download endpoint requires authentication. Format: `[Header-Name]: [Header Value]`.
5. Your crates are now in the local cargo cache. `cargo` will use the local copies
   rather than attempt to download them.

## Subsequent runs
1. If alternate registry dependencies have changed
   1. Run `cargo update -p [crate-names]` to update your `Cargo.lock` file.
   2. Run `cargo sideload --registry=[registry-name]` to download updated dependencies.
2. If alternate registry dependences have **not** changed, you don't have to do anything.

## Options
- `--force` will force `cargo sideload` to download a new copy of the specified `.crate` files.
  Previous copies of the `.crate` file for that crate will be deleted.

## Current restrictions
1. Authentication is currently hardcoded for GitLab's `PRIVATE-TOKEN` header.

## Remaining Work 
1. Validate crate file (e.g. ensure it's not just a 404 page)
2. Write tests.
3. Automatically run `cargo update` for registry dependencies that are in `Cargo.toml` but not `Cargo.lock`.
4. Deal with corrupt `.crate` files.
5. Improve error handling.
6. Improve console output.
