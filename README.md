**WARNING:** This crate makes it very easy to do dumb things with your access tokens, like put them in CI scripts.
Be careful.

`cargo-sideload` downloads crates from an alternative registry and stores them in your Cargo cache
as if they were downloaded by Cargo directly.

`cargo-sideload` makes a simple `GET` request with whatever headers you tell it to use. The primary use case is
downloading crates from an authenticated endpoint, a feature that Cargo does not currently support.
It is meant to be a temporary workaround until [this feature](https://github.com/rust-lang/rfcs/pull/2719) is added to Cargo.

[Cargo's documentation](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry) has lots
of useful information about working with alternative registries. 

# Installation
`cargo install cargo-sideload`

# First run
1. Add your alternate registry to `~/.cargo/config.toml`.
   ```toml
   [registries]
   test_registry = { index = "https://github.com/picklenerd/test_registry" }
   ```
2. Add `registry = "[registry-name]"` to any dependencies that use the registry.
   ```toml
   my_lib = { version = "1.0", registry = "test_registry" }
   ```
3. Run `cargo update` to let cargo sort out your dependencies and update your `Cargo.lock` file.
4. Run `cargo sideload --registry=[registry-name]` in your crate's root.
   - Use the `--headers` argument if your download endpoint requires authentication or other headers.  
   Header format: `[Header-Name]: [Header Value]`.
5. Your crates are now in the local cargo cache. `cargo` will use the local copies
   rather than attempt to download them.

# Subsequent runs
1. If alternate registry dependencies have changed
   1. Run `cargo update -p [crate-names]` to update your `Cargo.lock` file.
   2. Run `cargo sideload --registry=[registry-name]` to download updated dependencies.
2. If alternate registry dependences have **not** changed, you don't have to do anything.

# More Info
`cargo sideload --help` 

# Config file
Create the file `~/.config/cargo-sideload/config.toml`.

The config file can be used to set a default registry and to associate headers with specific registries.
This allows you to run the command as `cargo sideload` without providing `--registry` and `--headers` arguments. 

```
default_registry = "test_registry"
  
[registries.test_registry]
headers = [ "Authorization: Blah abcd1234" ] 

[registries.other_registry]
headers = [ 
        "PRIVATE-KEY: abcdef",
        "Some-Other-Header: And its value",
]
```

# Troubleshooting

`cargo-sideload` does not (currently) validate the crates that it downloads. If you type your
authentication header wrong, you might end up in a situation where your downloaded `.crate` file
is actually the HTML for a login page, or some similar situation.

`cargo-sideload` doesn't download crates that are already in the cache, even if those crates are corrupt.
If you find yourself in a situation where you want to force a new download, you can use the `--force` option.
This will delete the existing file and download a new copy.

If you try to run a normal Cargo command with a corrupt or otherwise invalid crate, 
you'll get an error message something like the one below. If that happens, you most likely need to troubleshoot
your download endpoint. Using `cargo sideload --debug --force` makes troubleshooting easier.

```
error: failed to download `my_lib v0.1.0 (registry `https://github.com/picklenerd/test_registry`)`

Caused by:
  unable to get packages from source

Caused by:
  failed to unpack package `my_lib v0.1.0 (registry `https://github.com/picklenerd/test_registry`)`

Caused by:
  failed to iterate over archive

Caused by:
  failed to fill whole buffer
```

## Remaining Work 
1. Validate crate file (e.g. ensure it's not just a login page).
2. Write tests.
3. Automatically run `cargo update` for registry dependencies that are in `Cargo.toml` but not `Cargo.lock`.
4. Improve console output.
5. Write a blog post explaining how to make an alternative registry using `git`, along with `cargo-sideload` and `cargo-index`.
