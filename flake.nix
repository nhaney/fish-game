{
    description = "Stay Off the Line! Remastered";

    inputs = {
        nixpkgs = {
            url = "nixpkgs/nixos-unstable";
        };
        flake-utils = {
            url = "github:numtide/flake-utils";
        };
    };

    outputs = { nixpkgs, flake-utils, ... }:
        flake-utils.lib.eachDefaultSystem(system:
            let
                pkgs = nixpkgs.legacyPackages.${system};
            in
            {
                packages = {
                    default = pkgs.callPackage ./default.nix {};
                };
                devShells.default = pkgs.mkShellNoCC rec {
                    packages = with pkgs; [
                        cargo
                        rustc

                        pkg-config

                        udev alsa-lib vulkan-loader
                        xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
                    ];

                    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
                };
            }
        );
    }
