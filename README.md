[![crates.io](https://img.shields.io/crates/v/cargo-sideload)](https://crates.io/crates/cargo-sideload)
[![docs.rs](https://docs.rs/cargo-sideload/badge.svg)](https://docs.rs/crate/cargo-sideload/)

**WARNING:** Be careful with your access tokens.

`cargo-sideload` is a toolkit for working with alternative Cargo registries. Its primary function is downloading
crates from authenticated download endpoints, a feature that Cargo does not currently support.
It is meant to be a temporary workaround until [this feature](https://github.com/rust-lang/rfcs/pull/2719) is added to Cargo.


[Cargo's documentation](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry) has lots
of useful information about working with alternative registries. 


# Installation
`cargo install cargo-sideload`


# Usage
1. Add your alternate registry to `~/.cargo/config.toml`.
   ```toml
   [registries]
   test_registry = { index = "https://github.com/picklenerd/test_registry" }
   ```
2. Add `registry = "[registry-name]"` to any dependencies that use the registry.
   ```toml
   my_lib = { version = "1.0", registry = "test_registry" }
   ```
3. Run `cargo sideload fetch --registry=[registry-name]` in your crate's root.
   - Use the `--headers` argument if your download endpoint requires authentication or other headers.  
   *Header format*: `[Header-Name]: [Header Value]`.
4. Your crates are now in the local Cargo cache. Running Cargo commands will work as usual. 
5. If you add or update dependencies from your private registry you'll have to run `cargo sideload fetch` again. 


# More Info
`cargo sideload --help` 


# Config file
A config file can be used to set a default registry and to associate headers with specific registries.
This allows you to run commands like `cargo sideload fetch` without providing `--registry` and `--headers` arguments. 

To use a config, create the file `~/.config/cargo-sideload/config.toml`. All of the available config options are
listed in the example below.

```toml
default_registry = "test_registry"
  
[registries.test_registry]
headers = [ "Authorization: Blah abcd1234" ] 

[registries.other_registry]
headers = [ 
        "PRIVATE-KEY: abcdef",
        "Some-Other-Header: And its value",
]
```

# Extra Tools
`cargo-sideload` comes with a few extra tools for working with private registries. These extra subcommands are provided
because existing tools don't always work with private registries or authenticated download endpoints.

`cargo sideload list [crate-name]` will list some information about each available version of the specified crate.
Yanked versions are not included in the result. Using `--latest` will print the info for the latest version of the crate,
while `--latest-version` will only return the latest version number.

`cargo sideload outdated --registry=[registry-name]` will list all dependencies with newer versions available 
in the specified registry. `--registry` is optional if you have a default registry set. A list of crates to check
can be specified with `--packages`.


# Troubleshooting

`cargo-sideload` uses the `pretty_env_logger` crate to print debug info. Use `RUST_LOG=debug cargo sideload fetch`
to see the details of the HTTP request and response for your file downloads. You will also see logs from Cargo and
any other dependencies based on the value of `RUST_LOG`. See the [env_logger](https://docs.rs/env_logger/0.8.2/env_logger/)
documentation for more details.

If you type your authentication header wrong, you might end up in a situation where your downloaded `.crate` file
is actually the HTML for a login page, or some similar situation. `cargo-sideload` will tell Cargo to unpack your 
`.crate` files after downloading them. If unpacking fails, you'll get an error and the downloaded file will be deleted.

If you find yourself in a situation where you want to force a new download, you can use the `--force` option.
This will delete the existing file and download a new copy.

If you try to run a normal Cargo command with a corrupt or otherwise invalid crate, 
you'll get an error message something like the one below. If that happens, you most likely need to troubleshoot
the download endpoint in your registry index or the headers in your request. Enabling logs and using the `--force` option 
can make this troubleshooting process much easier.


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
