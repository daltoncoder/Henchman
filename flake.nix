{
  description = "tee ai agent";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    let
      inherit (nixpkgs) lib;
      system = "x86_64-linux";

      pkgs = (
        import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: {
              # Upstream PR: https://github.com/NixOS/nixpkgs/pull/338278
              sgx-dcap-default-qpl = prev.stdenv.mkDerivation rec {
                pname = "sgx-dcap-default-qpl";
                version = "1.21";
                src = prev.fetchFromGitHub {
                  owner = "intel";
                  repo = "SGXDataCenterAttestationPrimitives";
                  rev = "dcap_${version}_reproducible";
                  hash = "sha256-2ZMu9F46yR4KmTV8Os3fcjgF1uoXxBT50aLx72Ri/WY=";
                  fetchSubmodules = true;
                };
                nativeBuildInputs = [ prev.pkg-config ];
                buildInputs = with prev; [
                  curl
                  openssl
                  boost
                  sgx-sdk
                ];
                preBuild = ''
                  source ${prev.sgx-sdk}/sgxsdk/environment
                '';
                makeFlags = [
                  "-C QuoteGeneration"
                  "qpl_wrapper"
                ];
                installPhase = ''
                  mkdir -p $out/lib
                  mv QuoteGeneration/build/linux/* $out/lib
                  ln -s $out/lib/libdcap_quoteprov.so $out/lib/libdcap_quoteprov.so.1
                  ln -s $out/lib/libsgx_default_qcnl_wrapper.so $out/lib/libsgx_default_qcnl_wrapper.so.1
                '';
              };
            })
          ];
        }
      );
      # inherit (pkgs) lib;
      craneLib = crane.mkLib pkgs;

      src = craneLib.path ./.;

      # Common arguments can be set here to avoid repeating them later
      commonArgs = {
        inherit src;
        strictDeps = true;
        pname = "tee-ai-agent";
        version = "0.1.0";
        nativeBuildInputs = with pkgs; [
          pkg-config
          # gcc
          # perl
          # cmake
          clang
          # protobuf
          # mold-wrapped

        ];
        buildInputs = with pkgs; [
          libclang
          # fontconfig
          # freetype
          # protobufc
          openssl
          sgx-dcap-default-qpl
          # curl
          # zstd
          # zlib
          # bzip2
          # lz4
          rocksdb
          # (snappy.override { static = true; })

          # For running nextest
          cacert
        ];
      } // commonVars;

      commonVars = {
        LIBCLANG_PATH = "${lib.getLib pkgs.libclang}/lib";
        ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
        DO_NOT_FORMAT = 1; # fix auto_generate_cdp
      };

      # Build *just* the cargo dependencies, so we can reuse all of that
      # work (e.g. via cachix or github artifacts) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly (commonArgs);
    in
    {
      # Allow using `nix flake check` to run tests and lints
      checks.${system} = {
        # Check formatting
        fmt = craneLib.cargoFmt {
          inherit (commonArgs) pname src;
          cargoExtraArgs = "--all";
        };

        # Check doc tests
        doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

        # Check clippy lints
        clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets --all-features -- -Dclippy::all -Dwarnings";
            CARGO_PROFILE = "dev";
          }
        );

        # Run tests with cargo-nextest
        nextest = craneLib.cargoNextest (
          commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestExtraArgs = "--workspace";
            RUST_LOG = "debug";
          }
        );
      };

      # Expose the node and services as packages
      packages.${system} = {
        default = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      };

      # Allow using `nix run` on the project
      apps.${system}.default = flake-utils.lib.mkApp { drv = self.packages.${system}.default; };

      # Allow using `nix develop` on the project
      devShells.${system}.default = craneLib.devShell (
        commonVars
        // {
          # Inherit inputs from checks
          checks = self.checks.${system};
          packages = with pkgs; [
            rust-analyzer
          ];
        }
      );

      # Allow using `nix fmt` on the project
      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
