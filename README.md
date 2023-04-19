# OVERLAY sales smart contract

This is the [Concordium](https://concordium.com/) smart contract modelling sales information of projects listed in
[OVERLAY](https://overlay.global/).

This smart contract module stores sales data of [OVERLAY](https://overlay.global/).

# How to build

## Prerequisite

You need to install the following tools to build this smart contract source codes.

1. [rustup](https://rustup.rs/)
2. [cargo-concordium](https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#cargo-concordium-testnet)

Please refer to the [Concordium official Quick start guide](https://developer.concordium.software/en/mainnet/smart-contracts/guides/quick-start.html)
for more information.

## Build

* Hit the following command to build.

```shell
# navigate to the ovl-sale-ccd-public project folder.
% cd ovl-sale-ccd-public

# run the build command.
% cargo concordium build
```

Then you can find wasm file built under the following directory.

```shell
% ls ./target/concordium/wasm32-unknown-unknown/release/ovl_sale_ccd_public.wasm.v1 
./target/concordium/wasm32-unknown-unknown/release/ovl_sale_ccd_public.wasm.v1
```

# How to run unit test

You can build and run unit tests by the following steps.

```shell
# navigate to the ovl-sale-ccd-public project folder.
% cd ovl-sale-ccd-public

# hit the following command to test your wasm modules with concordium-std/concordium-quickcheck features.
% cargo concordium test -- --features wasm-test

# you can also run the following test command mainly for non-wasm modules.
% cargo test
```

# LICENSE

see [LICENSE](./LICENSE) file.
