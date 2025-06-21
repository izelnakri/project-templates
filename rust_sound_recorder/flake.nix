{
  description = "Rust voice recorder with streams";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };
        unstable = import nixpkgs-unstable {
          inherit system;
          overlays = overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };
      in {
        # checks = {
        #   clippy = pkgs.writeShellApplication {
        #     name = "clippy-check";
        #     runtimeInputs = [ pkgs.rustc pkgs.cargo ];
        #     text = ''
        #       cargo clippy --all-targets --all-features -- -D warnings
        #     '';
        #   };
        #
        #   fmt = pkgs.writeShellApplication {
        #     name = "fmt-check";
        #     runtimeInputs = [ pkgs.rustfmt ];
        #     text = ''
        #       cargo fmt -- --check
        #     '';
        #   };
        #
        #   test = pkgs.writeShellApplication {
        #     name = "test-check";
        #     runtimeInputs = [ pkgs.rustc pkgs.cargo ];
        #     text = ''
        #       cargo test
        #     '';
        #   };
        #
        #   output = pkgs.nixosTest {
        #     name = "rust-output-test";
        #     nodes.machine = { config, pkgs, ... }: {
        #       environment.systemPackages = [ self.packages.${system}.default ];
        #       system.stateVersion = pkgs.lib.versions.majorMinor pkgs.lib.version;
        #     };
        #
        #     testScript = ''
        #       machine.wait_for_unit("default.target")
        #       machine.succeed("github_user_fetcher | grep -o \"GitHub User:\"")
        #
        #       machine.succeed("systemd-run --unit=github_user_fetcher github_user_fetcher --server")
        #       machine.wait_for_open_port(1234)
        #
        #       # machine.succeed("systemd-run --unit=github_user_fetcher_gui github_user_fetcher_gui")
        #
        #       # machine.succeed("pkg-config --exists github_user_fetcher")
        #       # machine.succeed("pkg-config --modversion github_user_fetcher")
        #     '';
        #   };
        # };

        devShells.default = import ./nix/devShells/default.nix {
          inherit pkgs unstable self system rustToolchain;
        };

        # devShells.default = pkgs.mkShell {
        #   buildInputs = [
        #     pkgs.rustc
        #     pkgs.cargo
        #     pkgs.rustfmt
        #     pkgs.clippy
        #     pkgs.pkg-config
        #     pkgs.openssl
        #   ];
        #   shellHook = ''
        #     echo "Welcome to the Rust Dev Shell"
        #   '';
        # };

        formatter = pkgs.nixpkgs-fmt;

        # apps = {
        #   default = {
        #     type = "app";
        #     program = "${self.packages.${system}.default}/bin/github_user_fetcher";
        #   };
        # };
        #
        # packages = rec {
        #   # default = rustPlatform.buildRustPackage {
        #   #   pname = "rust_app_binary";
        #   #   version = "0.1.0";
        #   #   src = ./.;
        #   #   cargoLock.lockFile = ./Cargo.lock;
        #   #   nativeBuildInputs = [ pkgs.pkg-config ];
        #   #   buildInputs = [ pkgs.openssl ];
        #   #   doCheck = true;
        #   #   checkPhase = ''
        #   #     cargo test
        #   #   '';
        #   #   postInstall = ''
        #   #     echo "Installed rust_app_binary to $out/bin"
        #   #   '';
        #   #   meta = {
        #   #     description = "An example Rust binary built with Nix flakes";
        #   #     license = pkgs.lib.licenses.mit;
        #   #   };
        #   # };
        #
        #   # development = default;
        #
        #   # production = default.overrideAttrs (old: {
        #   #   cargoBuildFlags = [ "--release" ];
        #   # });
        #
        #   dockerImage = pkgs.dockerTools.buildLayeredImage {
        #     name = "rust_app_binary";
        #     tag = "dev";
        #     created = "now";
        #     contents = [
        #       pkgs.zsh
        #       pkgs.coreutils
        #       pkgs.cacert
        #       self.packages.${system}.default
        #     ];
        #     config = {
        #       Env = [
        #         "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        #       ];
        #       Cmd = [ "/bin/rust_app_binary" ];
        #     };
        #   };
        #
        #   dockerProductionImage = pkgs.dockerTools.buildLayeredImage {
        #     name = "rust_app_binary";
        #     tag = "prod";
        #     created = "now";
        #     contents = [
        #       pkgs.zsh
        #       pkgs.coreutils
        #       pkgs.cacert
        #       self.packages.${system}.production
        #     ];
        #     config = {
        #       Env = [
        #         "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        #       ];
        #       Cmd = [ "/bin/rust_app_binary" ];
        #     };
        #   };
        # };
      }
    );
}
