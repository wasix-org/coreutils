# coreutils for WASIX

Based on uutils/coreutils main at `a6d1eb3835c0f808fa9678e4551df7377bcab8d3`
(0.13.0). The Wasmer package version is 1.0.27 to continue the existing
`wasmer/coreutils` release series.

Build with Rustup, Python 3.11+, Wasmer 7.4.2, wasm-tools 1.251.0, and the pinned
WASIX tools:

```sh
rustup toolchain install 1.96.1 --profile minimal
cargo +1.96.1 install cargo-wasix --version 0.1.33 --locked
cargo wasix download-toolchain v2026-07-07.3+rust-1.96
ln -s "$(rustup which --toolchain 1.96.1 cargo)" "$(rustc +wasix --print sysroot)/bin/cargo"
bash wasix/build.sh
python3 wasix/test.py
```

The build runs `cargo wasix build --release --locked --no-default-features
--features feat_wasix --bin coreutils`. Cargo-wasix uses Binaryen 130 to
convert exceptions for the browser. The committed lockfile resolves through
the WASIX registry; source paths are normalized for repeatable builds.

The multicall module exports upstream's WASI utility set plus `env` and
`nohup`. `wasmer.toml` is generated from the compiled binary's `--list`
output, and tests execute every exported command. Utilities requiring other
platform facilities (for example SELinux, utmp, or Linux mount tables) are
not advertised as working commands. `nohup` uses WASIX signals, descriptor
redirection, and spawn-and-wait; `tail -f` and `tail -F` use upstream's polling watcher.
Wasmer's exec API does not carry ignored signals, so `nohup` waits for a
spawned child and returns its exit code. Tail includes creation time when
comparing WASIX file identities because Wasmer derives inodes from paths.
Log rotation must happen inside the sandbox; renaming mounted files from
outside it does not invalidate Wasmer's cached guest metadata.

Artifacts are `.wasix/coreutils.wasm` and `.wasix/coreutils-1.0.27.webc`.
Dependency license notices are included in the package. The package contains
no shell wrappers.

For a clean reproducibility check, save the first WebC, set
`CARGO_TARGET_DIR=.wasix/repro-target`, run `bash wasix/build.sh` again, and
compare the two WebCs with `cmp`.

```sh
wasmer run .wasix/coreutils-1.0.27.webc --entrypoint tail -- -n 5 /workspace/log
wasmer publish . --registry wasmer.io --wait=container --non-interactive
wasmer publish . --registry wasmer.wtf --wait=container --non-interactive
```

Package 1.0.27 stores dependency notices at `/opt/coreutils/licenses`. This keeps
license mounts out of `/usr`, where they interfere with Wasmer command installation.
The compiled utility versions are unchanged.
