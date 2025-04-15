{
  description = "GitHub user fetcher in C using libcurl and jansson";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = 
    { self, nixpkgs }: 
    let 
      allSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; };
      });
    in {
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            meson
            ninja
            pkg-config
            curl.dev
            jansson
            gcc
          ];

          PKG_CONFIG_PATH = pkgs.lib.makeLibraryPath [ pkgs.jansson.dev pkgs.curl.dev ] + "/lib/pkgconfig";

          shellHook = with pkgs; ''
            # export CFLAGS=$(pkg-config --cflags libcurl jansson)
            # export LDFLAGS=$(pkg-config --libs libcurl jansson)

            echo "🔧 Dev environment ready. Run: make"
          '';
        };
      });
    };
}
