{
  description = "tee ai agent";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixsgx = {
      url = "github:matter-labs/nixsgx";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = (
        import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.nixsgx.overlays.default ];
        }
      );
      inherit (pkgs) lib;

      craneLib = inputs.crane.mkLib pkgs;
      src = craneLib.path ./.;

      # Common arguments can be set here to avoid repeating them later
      commonArgs = {
        inherit src;
        strictDeps = true;
        pname = "tee-ai-agent";
        version = "0.1.0";
        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
        ];
        buildInputs = with pkgs; [
          libclang
          openssl
          rocksdb
          cacert
          nixsgx.sgx-dcap.default_qpl
        ];
      } // commonVars;

      commonVars = {
        LIBCLANG_PATH = "${lib.getLib pkgs.libclang}/lib";
        ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
        DO_NOT_FORMAT = 1; # fixup auto_generate_cdp
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

      packages.${system} = rec {
        default = tee-ai-agent;

        # Raw tee ai agent binary derivation
        tee-ai-agent = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
          }
        );

        # Minimal docker image for the agent
        docker = pkgs.dockerTools.buildImage {
          name = "tee-ai-agent";
          tag = "latest";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ tee-ai-agent ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Cmd = [ "/bin/tee_ai_agent" ];
          };
        };

        # Bundled rootfs directory containing /bin and /nix with all hard-linked dependencies
        bundle = pkgs.stdenvNoCC.mkDerivation {
          name = "bundled-tee-ai-agent";
          src = docker;
          patchPhase = null;
          buildPhase = ''mkdir -p $out && tar -xvf layer.tar -C $out "nix" "./bin"'';
        };

        # Reproducable gramine docker runtime for tee ai agent
        gramine-docker = lib.tee.sgxGramineContainer {
          name = "gramine-tee-ai-agent";
          tag = "latest";

          packages = [ tee-ai-agent ];
          entrypoint = "${tee-ai-agent}/bin/tee_ai_agent";

          manifest = {
            loader = {
              log_level = "debug";
              env = {
                RUST_LOG.passthrough = true;
              };
            };

            sgx = {
              edmm_enable = false;
              enclave_size = "4G";
              max_threads = 16;
            };
          };

          # TODO: qcnl config? Currently fallback to using default intel v4 collateral endpoint
          # sgx_default_qcnl_conf = '' ... '';

          # TODO: Optionally use a signature file for the mrenclave, though we might not need anything for `mrsigner`.
          #       For now, we fallback to the dummy testing key provided by nixsgx.
          # sigFile = ./signature;
        };
      };

      # Allow using `nix develop` on the project
      devShells.${system}.default = craneLib.devShell (
        commonVars
        // {
          # Inherit inputs from checks
          checks = self.checks.${system};
          packages = with pkgs; [
            rust-analyzer
            dive # inspect docker images
          ];
        }
      );

      # Allow using `nix fmt` on the project
      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
