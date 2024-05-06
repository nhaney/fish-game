{
    makeRustPlatform,
    rust-bin,
    pkg-config,
    udev,
    alsa-lib,
    vulkan-loader,
    vulkan-tools,
    vulkan-headers,
    vulkan-validation-layers,
    xorg,
    libxkbcommon,
    clang,
    mold,
    wasm-bindgen-cli,
    binaryen
}:
let
    rust = rust-bin.nightly.latest.default.override { extensions = ["rust-src" ]; targets = ["wasm32-unknown-unknown"]; };
    rustPlatform = makeRustPlatform { cargo = rust; rustc = rust; };

    appNativeBuildInputs = [ pkg-config clang mold ];
    appWasmNativeBuildInputs = appNativeBuildInputs ++ [ wasm-bindgen-cli binaryen ];

    appRuntimeInputs = [ vulkan-loader xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ];

    appBuildInputs = appRuntimeInputs ++ [ udev alsa-lib xorg.libX11 vulkan-tools vulkan-headers vulkan-validation-layers ];

in
{
    native = rustPlatform.buildRustPackage {
        pname = "fish-game";
        version = "1.0.0";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = appNativeBuildInputs;
        buildInputs = appBuildInputs;

        # TODO: Remove dynamic linking feature in build so that the game can be distributed as a single executable.
        # patch = ''
        #     substituteInPlace ./Cargo.toml 
        #         --replace "bevy = { version = \"0.13.2\", features = [\"dynamic_linking\"] }" "bevy = \"0.13.2\""
        # '';

        postInstall = ''
            cp -r assets $out/bin
        '';
    };

    wasm = rustPlatform.buildRustPackage {
        pname = "fish-game";
        version = "1.0.0";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = appWasmNativeBuildInputs;
        buildInputs = appBuildInputs;

        # TODO: Remove dynamic linking feature in build so that the game can be distributed as a single executable.
        # patch = ''
        #     substituteInPlace ./Cargo.toml 
        #         --replace "bevy = { version = \"0.13.2\", features = [\"dynamic_linking\"] }" "bevy = \"0.13.2\""
        # '';

        # Custom build phase that uses the wasm target.
        # TODO: See if we can do this without overriding.
        buildPhase = ''
            cargo build --profile wasm-release --target wasm32-unknown-unknown

            echo 'Creating out dir...'
            mkdir -p $out/bin

            echo 'Generating JS code to run the WASM...'
            wasm-bindgen --no-typescript --out-dir $out/bin --target web target/wasm32-unknown-unknown/wasm-release/fish-game.wasm

            echo 'Optimizing WASM binary...'
            wasm-opt -Oz --output optimized.wasm $out/bin/fish-game_bg.wasm
            mv optimized.wasm $out/bin/fish-game_bg.wasm

            echo 'Copying assets into output directory...'
            cp -r assets $out/bin
        '';

        installPhase = "echo 'Skipping installPhase in web build.'";
        checkPhase = "echo 'Skipping checkPhase in web build.'";
    };
}
