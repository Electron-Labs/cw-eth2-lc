# cw-eth2-lc

### Tests

#### Unit tests

```sh
cargo test -- --nocapture
```

#### E2E tests
Make sure you have the [prerequisites](#prerequisites) installed before attempting this.

To run e2e tests, you need to run the [dev server](#dev-server) before running the following command.

```sh
# builds the smart contract and adds the artifacts in ./artifacts folder
./scripts/build_sc.sh

cargo test --features e2e -- --nocapture --test-threads 1
```
### Prerequisites

#### Cosmwasm-check
Used in `./scripts/build_sc.sh` to perform various checks on the wasm file to make sure its valid - such as if the wasm file has floating point operations .etc.
```sh
cargo install cosmwasm-check
```

#### Wasmd CLI
CLI to interact with the wasmd node - refer to https://github.com/CosmWasm/wasmd#quick-start for installation.


### Dev server
We will be running the e2e tests on wasmd node locally hosted using docker.
```sh
# Add a key with the followin mnemonic into your operating system wallet, this address will be used to call the contract during testing to pay transaction fee .etc.
# come fury another excite blue obtain throw rhythm enjoy pulse olive damage tomato mention patrol farm robot diesel doll execute vapor more theme flee
wasmd keys add cw-eth2-lc --recover

# remove exsisting docker volume if any
docker volume rm -f wasmd_data

# creates the docker volume and sets it up with the initial state
docker run --rm -it \
    -e PASSWORD=xxxxxxxxx \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/setup_wasmd.sh wasm15mf38plfzfzgp6r3jv8l335a87fnxe2684x269

# runs the wasmd node
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 -p 9090:9090 \
    --mount type=volume,source=wasmd_data,target=/root \
    cosmwasm/wasmd:latest /opt/run_wasmd.sh
```

To reset the state run the last 3 command, to restart the node run only the last command.
