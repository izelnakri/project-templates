{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    clang
    curl
    pkg-config
    jansson
  ];

  shellHook = ''
    echo "✨ Dev shell ready! Run: make"
  '';
}
