{
  description = "tee ai agent";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # Reproducible 
    nixsgx.url = "github:matter-labs/nixsgx";
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
            name = "tee-ai-agent-root";
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

        # Docker image providing a reproducible gramine environment for tee ai agent enclave
        #
        # ## Usage
        #
        # Build and load image into docker:
        #
        # ```
        # nix build .\#gramine-docker
        # sudo docker load < result
        # ```
        #
        # Run container:
        #
        # ```
        # sudo docker run -i --init --rm \
        #     -p 6969:6969 -p 8000:8000 \
        #     --device /dev/sgx_enclave \
        #     -v /var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
        #     gramine-tee-ai-agent:latest
        # ```
        #
        # View mrenclave/signature struct
        #
        # ```
        # sudo docker run -i --init --rm \
        #     gramine-tee-ai-agent:latest \
        #     "gramine-sgx-sigstruct-view app.sig"
        # ```
        gramine-docker = lib.tee.sgxGramineContainer rec {
          appName = "tee-ai-agent";
          name = "gramine-tee-ai-agent";
          tag = "latest";
          maxLayers = 125;

          # Trusted packages to include in MRENCLAVE
          packages = [
            tee-ai-agent
            pkgs.chromium
          ];
          entrypoint = "${tee-ai-agent}/bin/tee_ai_agent";

          # Enclave manifest configuration. Trusted files and other options are automatically
          # setup based on packages, their dependencies, and provided entrypoint.
          manifest = {
            loader = {
              log_level = "debug";
              env = {
                # Allow setting rust log from the host
                RUST_LOG.passthrough = true;
                # Hardcode headless-chrome to use our included version
                CHROME = "${pkgs.lib.getExe pkgs.chromium}";
              };
            };
            sgx = {
              edmm_enable = false;
              enclave_size = "4G";
              max_threads = 16;
            };
          };

          # Untrusted utility packages and startup script
          extendedPackages = [ ];
          extraCmd = ''echo "Starting ${name}"; is-sgx-available; gramine-sgx-sigstruct-view ${appName}.sig'';

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
