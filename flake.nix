{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        packages = rec {
          default = pkgs.rustPlatform.buildRustPackage rec {
            pname = "omglol-ddns";
            version = "0.1.0";
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
            buildInputs =
              if system == "aarch64-darwin" || system == "x86_64-darwin" then
                [ pkgs.darwin.apple_sdk.frameworks.Security ]
              else
                [ ];
          };
          service = import ./service { omglol-ddns = default; };
        };
      });
}
