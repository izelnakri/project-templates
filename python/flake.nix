{
  description = "Hello World Python app with uv";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        python = pkgs.python312Full;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [ python pkgs.uv ];
          shellHook = ''
            echo "🐍 Python: $(python3 --version)"
            echo "⚡ uv: $(uv --version)"
          '';
        };
      }
    );
}
