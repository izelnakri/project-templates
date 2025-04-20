{
  description = "GitHub user fetcher in C using libcurl and jansson";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = 
    { self, nixpkgs }: 
    let 
      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
          f { inherit pkgs system; }
      );
    in {
      devShells = forAllSystems ({ pkgs, system }: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            meson
            ninja
            pkg-config
            curl.dev
            jansson
            criterion
            gtk4
          ];

          PKG_CONFIG_PATH = pkgs.lib.makeLibraryPath [ pkgs.jansson.dev pkgs.curl.dev ] + "/lib/pkgconfig";

          shellHook = with pkgs; ''
            # export CFLAGS=$(pkg-config --cflags libcurl jansson)
            # export LDFLAGS=$(pkg-config --libs libcurl jansson)

            echo "🔧 Dev environment ready. Run: make"
          '';
        };
      });

      packages = forAllSystems ({ pkgs, system }: {
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
