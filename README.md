ican
====

```sh
Generate Rust type definitions for a canister on the fly

Usage: ican [OPTIONS] --canister <CANISTER>

Options:
  -c, --canister <CANISTER>  The canister principal to fetch type definitions for
  -t, --target <TARGET>      The output format for the type definitions [default: agent] [possible values: agent, canister]
  -p, --path <PATH>          Path to store the generated types [default: canister/def.rs]
  -h, --help                 Print help
  -V, --version              Print version
```