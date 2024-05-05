{
    description = "Stay Off the Line! Remastered";

    inputs = {
        nixpkgs = {
            url = "nixpkgs/nixos-unstable";
        };
        flake-utils = {
            url = "github:numtide/flake-utils";
        };
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
        flake-utils.lib.eachDefaultSystem(system:
            let
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs { inherit system overlays; };
            in
            {
                packages = {
                    default = pkgs.callPackage ./default.nix {};
                };
                devShells.default = pkgs.mkShellNoCC rec {
                    packages = with pkgs; [
                        (rust-bin.beta.latest.default.override {
                            extensions = [ ];
                            targets = ["wasm32-unknown-unknown"];
                        })

                        pkg-config

                        udev alsa-lib vulkan-loader
                        xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon

                        clang mold
                    ];

                    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
                };
            }
        );
    }
