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
}:
let
    rust = rust-bin.nightly.latest.default.override { extensions = ["rust-src" ]; };
    rustPlatform = makeRustPlatform { cargo = rust; rustc = rust; };

    appNativeBuildInputs = [ pkg-config clang mold ];

    appRuntimeInputs = [ vulkan-loader xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ];

    appBuildInputs = appRuntimeInputs ++ [ udev alsa-lib xorg.libX11 vulkan-tools vulkan-headers vulkan-validation-layers ];
in
rustPlatform.buildRustPackage {
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
}
