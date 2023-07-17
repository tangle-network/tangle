## How to use the scripts

This a simple readme file that will outline the required steps to run the included scripts.


### Run a standalone Tangle Network

For running a standalone tangle network, Here are the steps that you need to follow:

1. Compile the standalone in the `release` mode:
```sh
cargo b -rp tangle-standalone
```
2. Execute the `run-standalone.sh` script:
```sh
./scripts/run-standalone --clean
```

Note that this will start a clean network state, if you want to continue running on an old state (using old database)
just omit the `--clean` flag.
