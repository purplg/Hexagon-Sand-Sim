run:
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.x86_64.env

    cargo run

build: export-svg
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.x86_64.env

    cargo build

watch:
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.x86_64.env

    cargo watch -d 1 --clear -x lbuild

clean:
    rm -rf build/

wasm: clean export-svg
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.wasm.env

    mkdir -p build/
    cp -r wasm/* build/.
    cp -r assets/ build/assets
    cargo build --release --target wasm32-unknown-unknown
    cp $CARGO_TARGET_DIR/wasm32-unknown-unknown/release/sandsim.wasm build/sandsim.wasm
    wasm-bindgen --no-typescript --out-name sandsim --out-dir build --target web build/sandsim.wasm

    sfz ./build

export-svg:
    inkscape --export-filename assets/hex.png resources/hex.svg
