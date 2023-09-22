# cw-eth2-lc

### Tests

#### Unit tests

```sh
cargo test -- --nocapture
```

#### E2E tests

```sh
# builds the smart contract and adds the artifacts in ./artifacts folder
cargo test --features e2e -- --nocapture --test-threads 1
```
### Build Prerequisites

#### Cosmwasm-check
Used in `./scripts/build_sc.sh` to perform various checks on the wasm file to make sure its valid - such as if the wasm file has floating point operations .etc.
```sh
cargo install cosmwasm-check
```
