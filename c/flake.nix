{
  description = "GitHub User Fetcher - Example Complex C binary & library that uses external dependencies & tools";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, nixpkgs-unstable, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        unstable = nixpkgs-unstable.legacyPackages.${system};
        getAllPkgConfigPaths = pkgs: inputs:
          let
            allDeps = pkgs.lib.closePropagation inputs;
          in
          pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "out" "share/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" allDeps;
      in
      {
        checks = {
          # $ nix flake check # runs all checks $ nix run .#checks.x86_64-linux.cppcheck # runs specific check
          clang-tidy = pkgs.writeShellApplication {
            name = "clang-tidy-check";
            runtimeInputs = [ pkgs.clang-tools pkgs.meson pkgs.ninja ];
            text = ''
              set -eux
              meson setup build 
              make lint-clang-tidy
            '';
          };

          cppcheck = pkgs.writeShellApplication {
            name = "cppcheck-check";
            runtimeInputs = [ pkgs.cppcheck ];
            text = "make lint-cppcheck";
          };

          flawfinder = pkgs.writeShellApplication {
            name = "flawfinder-check";
            runtimeInputs = [ pkgs.flawfinder ];
            text = "make lint-flawfinder";
          };

          format = pkgs.writeShellApplication {
            name = "format-check";
            runtimeInputs = [ pkgs.clang-tools ];
            text = "make format-check";
          };

          test = pkgs.writeShellApplication {
            name = "test-check";
            runtimeInputs = [ pkgs.clang-tools pkgs.meson pkgs.ninja ];
            text = "make test";
          };

          output = pkgs.nixosTest {
            # NOTE: running this in python with: $ nix run '.#checks.x86_64-linux.output.driverInteractive' # inside python shell: run_tests()
            name = "output-test"; # NOTE: run this on nix flake check or $ nix run .#checks.x86_64-linux.output
            nodes.machine = { config, pkgs, ... }: {
              environment.systemPackages = [
                pkgs.pkg-config-unwrapped
                self.packages.${system}.default
              ];
              environment.variables = {
                PKG_CONFIG_PATH = "$PKG_CONFIG_PATH:${self.packages.${system}.default}/lib/pkgconfig";
              };
              system.stateVersion = pkgs.lib.versions.majorMinor pkgs.lib.version;
            };

            testScript = ''
              machine.wait_for_unit("default.target")
              machine.succeed("github_user_fetcher | grep -o \"GitHub User:\"")
              
              machine.succeed("systemd-run --unit=github_user_fetcher github_user_fetcher --server")
              machine.wait_for_open_port(1234)

              machine.succeed("systemd-run --unit=github_user_fetcher_gui github_user_fetcher_gui")

              machine.succeed("pkg-config --exists github_user_fetcher")
              machine.succeed("pkg-config --modversion github_user_fetcher")
            '';
          };
        };

        devShells.default = import ./nix/devShells/default.nix {
          inherit pkgs unstable self system;
        };

        formatter = pkgs.nixpkgs-fmt;

        apps = rec {
          cli = {
            type = "app"; # NOTE: in future of nix, maybe "docker", "service", "shell", # check if this is required
            program = "${self.packages.${system}.default}/bin/github_user_fetcher";
          };
          gui = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/github_user_fetcher_gui";
          };
          default = cli;
        };

        packages = rec {
          default = pkgs.stdenv.mkDerivation {
            # $ nix profile install . # installs it
            name = "github_user_fetcher";
            src = ./.;
            nativeBuildInputs = [ pkgs.meson pkgs.ninja pkgs.pkg-config ];
            buildInputs = [ pkgs.curl.dev pkgs.jansson pkgs.criterion pkgs.gtk4 ];

            installPhase = ''
              runHook preInstall
              ninja -C . install
              runHook postInstall
            '';

            setupHook = pkgs.writeText "setup-hook.sh" ''
              export PKG_CONFIG_PATH="''${PKG_CONFIG_PATH-}''${PKG_CONFIG_PATH:+:}$out/lib/pkgconfig"
            '';

            passthru.pkgconfigPath = "$out/lib/pkgconfig";
          };

          development = default;

          production = default.overrideAttrs (old: {
            mesonBuildType = "release";
          });

          dockerImage = pkgs.dockerTools.buildLayeredImage {
            # this can be the production image, probably make streamedPaths?
            name = "github_user_fetcher";
            tag = "dev";
            created = "now";

            contents = [
              pkgs.zsh
              pkgs.coreutils
              pkgs.pkg-config-unwrapped
              pkgs.cacert
              pkgs.fontconfig
              pkgs.dejavu_fonts
              self.packages.${system}.default
            ];

            config = {
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "XDG_DATA_DIRS=${pkgs.gtk4}/share"
                "PKG_CONFIG_PATH=/lib/pkgconfig"
              ];
              Cmd = [ "/bin/github_user_fetcher" ];
            };
          };

          dockerProductionImage = pkgs.dockerTools.buildLayeredImage {
            name = "github_user_fetcher";
            tag = "prod";
            created = "now";

            contents = [
              pkgs.zsh
              pkgs.coreutils
              pkgs.pkg-config-unwrapped
              pkgs.cacert
              pkgs.fontconfig
              pkgs.dejavu_fonts
              self.packages.${system}.production
            ];

            config = {
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "XDG_DATA_DIRS=${pkgs.gtk4}/share"
                "PKG_CONFIG_PATH=/lib/pkgconfig"
              ];
              Cmd = [ "/bin/github_user_fetcher" ];
            };
          };
        };
      }
    );
}
