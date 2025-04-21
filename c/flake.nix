{
  description = "GitHub user fetcher in C using libcurl and jansson";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = 
    { self, nixpkgs, nixpkgs-unstable }: 
    let 
      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          unstable = import nixpkgs-unstable { inherit system; };
        in
          f { inherit pkgs system unstable; }
      );
      getAllPkgConfigPaths = pkgs: inputs:
        let
          allDeps = pkgs.lib.closePropagation inputs;
        in
          pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "out" "share/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" allDeps + ":" +
          pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" allDeps;
    in {
      devShells = forAllSystems ({ pkgs, system, unstable }: {
        default = pkgs.mkShell rec {
          nativeBuildInputs = [];

          buildInputs = with pkgs; [
            meson
            ninja
            pkg-config-unwrapped
            unstable.vcpkg
            curl.dev
            jansson
            criterion
            gtk4
          ];
          
          # NOTE: These were probably redundant:
          # VCPKG_ROOT = "${unstable.vcpkg}/share/vcpkg";
          # VCPKG_FORCE_SYSTEM_BINARIES = 1;

          PKG_CONFIG_PATH = getAllPkgConfigPaths pkgs (buildInputs ++ nativeBuildInputs);

          shellHook = with pkgs; ''
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:$HOME/.cache/vcpkg/packages/benchmark_x64-linux/lib/pkgconfig"

            # TODO: In future add vcpkg packages(ports) directly to /nix/store
            vcpkg install

            echo "🔧 Dev environment ready. Run: make"
          '';
        };
      });

      packages = forAllSystems ({ pkgs, system, unstable }: {
        default = pkgs.callPackage ./default.nix {};

        dockerImage = pkgs.dockerTools.buildImage {
          name = "github_user_fetcher";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "c-image-root";
            pathsToLink = [ "/bin" ];
            paths = [ 
              pkgs.cacert 
              pkgs.fontconfig
              pkgs.dejavu_fonts
              self.packages.${system}.default 
            ];
          };
          runAsRoot = "true";
          # extraCommands, runAsRoot, created, fromImage(nix construct), maxLayers, compressor = "zstd", architecture
          # contents = [(writeTextDir file content)] (for buildLayeredImage)
          # fakeRootCommands

          # streamNixShellImage{ name, tag, drv(exact mkShell), command, run }

          config = {
            Env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              "XDG_DATA_DIRS=${pkgs.gtk4}/share"
            ];
            # ExposedPorts{}, WorkingDir, Volumes{}, Healthcheck{ test[], Interval, Timeout, Retries }
            # Cmd = [ "/bin/github_user_fetcher" ]; # NOTE: Try with gui too
          };
        };
      });
      # # Used to demonstrate how virtualisation.oci-containers.imageStream works
      # nginxStream = pkgs.dockerTools.streamLayeredImage nginxArguments; # includeNixDB
      # pullImage, buildImageWithNixDb, buildLayeredImage (has contents)
    };
}
