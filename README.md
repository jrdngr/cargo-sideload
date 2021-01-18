Download dependencies that cargo won't, such as those that require authentication.

## How to use
1. Add your alternate registry to `~/.cargo/config`
2. Add `registry = "[registry-name]"` to any dependencies that use the registry
3. Run `cargo update` 
4. Run `cargo sideload --registry=[registry-name]` in your crate's root
   - Set `CARGO_SIDELOAD_ACCESS_TOKEN` in your shell or use the `--access-token` 
   flag if your download endpoint requires authentication.
5. Your crates are now in the local cargo cache. `cargo` will use the local copies
   rather than attempt to download them.

## Current restrictions
1. Authentication is currently hardcoded for GitLab

## Remaining Work
1. Generalize authentication
