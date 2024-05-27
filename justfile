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
    rm -rf wasm/

_wasm-prepare: clean export-svg
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.wasm.env

    mkdir -p wasm/
    cp -r html/* wasm/.
    cp -r assets wasm/

_wasm-bindgen profile:
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.wasm.env

    wasm-bindgen \
      --no-typescript \
      --out-name sandsim \
      --out-dir wasm \
      --target web $CARGO_TARGET_DIR/wasm32-unknown-unknown/{{ profile }}/sandsim.wasm

wasm-release: _wasm-prepare
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.wasm.env

    cargo build --profile wasm-release --target wasm32-unknown-unknown

    just _wasm-bindgen wasm-release

    wasm-opt -Oz --output /tmp/sandsim.wasm wasm/sandsim_bg.wasm
    mv /tmp/sandsim.wasm wasm/sandsim_bg.wasm

wasm: _wasm-prepare
    #!/usr/bin/env sh
    set -euxo pipefail
    source ./.wasm.env

    cargo build --release --target wasm32-unknown-unknown
    just _wasm-bindgen release

serve-wasm: wasm
    sfz ./wasm

export-svg:
    inkscape --export-filename assets/hex.png resources/hex.svg
