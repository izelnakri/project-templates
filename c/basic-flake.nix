{
  description = "Meson + vcpkg + civetweb + Nix dev env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Where vcpkg will live
        vcpkgDir = pkgs.stdenv.mkDerivation {
          name = "vcpkg-with-civetweb";
          src = pkgs.fetchFromGitHub {
            owner = "microsoft";
            repo = "vcpkg";
            rev = "2024.02.14"; # Lock to known-good commit
            sha256 = "sha256-g7iO13zCtINSmkWo0X9aMw9DQ1DE2MY5ZfDtv09D0dU=";
          };
          buildPhase = ''
            ./bootstrap-vcpkg.sh
            ./vcpkg install civetweb
          '';
          installPhase = ''
            mkdir -p $out
            cp -r . $out/
          '';
        };

        pkgConfigPath = "${vcpkgDir}/installed/x64-linux/lib/pkgconfig";

        nativeFile = pkgs.writeText "vcpkg-native.ini" ''
          [binaries]
          pkgconfig = '${pkgs.pkg-config}/bin/pkg-config'

          [paths]
          pkg_config_path = '${pkgConfigPath}'
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.meson
            pkgs.ninja
            pkgs.pkg-config
            pkgs.gcc
          ];

          shellHook = ''
            export VCPKG_ROOT=${vcpkgDir}
            export PKG_CONFIG_PATH=${pkgConfigPath}
            echo "Vcpkg with civetweb is set up!"
          '';
        };

        # Optional build target with nix build .#defaultPackage.x86_64-linux
        packages.default = pkgs.stdenv.mkDerivation {
          name = "github_user_fetcher";
          src = ./.;

          nativeBuildInputs = [ pkgs.meson pkgs.ninja pkgs.pkg-config ];
          buildInputs = [ pkgs.gcc ];

          configurePhase = ''
            meson setup build --native-file ${nativeFile}
          '';

          buildPhase = "ninja -C build";
          installPhase = ''
            mkdir -p $out/bin
            cp build/github_user_fetcher $out/bin/
          '';
        };
      });
}
