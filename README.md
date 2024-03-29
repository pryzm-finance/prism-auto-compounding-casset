# Prism Auto Compounding Liquid Staking Contracts

This monorepository contains the source code for the smart contracts implementing bAsset Protocol on the [Terra](https://terra.money) blockchain.

You can find information about the architecture, usage, and function of the smart contracts on the official Prism documentation [site]().


## Contracts
| Contract        | Reference                                              | Description                                                                                                                        |
|-----------------| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| [`prism-hub`]() |[doc](https://github.com/prism-finance/prism-auto-compounding-casset/blob/main/contracts/prism_hub/README.md)| Manages minted cLunas and bonded Lunas and auto compound them

## Development

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit tests embedded within the contract source directories. You can run:

```sh
cargo test unit-test
cargo test integration-test
```

### Compiling

After making sure tests pass, you can compile each contract with the following:

```sh
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw1_subkeys.wasm .
ls -l cw1_subkeys.wasm
sha256sum cw1_subkeys.wasm
```

#### Production

For production builds, run the following:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.5
```

This performs several optimizations which can significantly reduce the final size of the contract binaries, which will be available inside the `artifacts/` directory.

## License

Copyright 2022 Prism Protocol

Licensed under the GNU GENERAL PUBLIC LICENSE Version 3 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at https://www.gnu.org/licenses/gpl-3.0.en.html. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

See the License for the specific language governing permissions and limitations under the License.
