{
  description = "Flow3r SIP dev environment with Naersk";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        python = pkgs.python3;
        pythonPkgs = python.pkgs;
      in
      {
        # Host SIP client crate
        flow3r-sip-host = naersk-lib.buildPackage ./flow3r-sip-host;

        # Badge-native SIP crate
        flow3r-sip-native = naersk-lib.buildPackage ./flow3r-sip-native;

        # Development shell
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              python
              pythonPkgs.pygame
              wasm-pack
              git
              pkg-config # Needed by alsa-sys
              alsa-lib # Provides ALSA headers and lib
            ];

            shellHook = ''
              echo "Flow3r SIP devShell ready!"

              # Python virtualenv for simulator
              if [ ! -d venv ]; then
                python -m venv venv
                source venv/bin/activate
                pip install --upgrade pip pygame wasmer wasmer-compiler-cranelift
              fi
            '';
          };
      }
    );
}
