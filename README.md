# overlay_sales


# Unit Tests

You can build and run unit tests by the following steps.

```shell
# navigate to the ovl-sale-ccd-public project folder.
% cd ovl-sale-ccd-public

# hit the following command to test your wasm modules with concordium-std/concordium-quickcheck features.
% cargo concordium test -- --features wasm-test

# you can also run the following test command mainly for non-wasm modules.
% cargo test
```
