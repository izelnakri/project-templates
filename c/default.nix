{ stdenv, fetchFromGitHub, meson, ninja, pkg-config, curl, jansson, criterion, gtk4 }:

stdenv.mkDerivation {
  name = "github_user_fetcher";
  src = ./.;
  nativeBuildInputs = [ meson ninja pkg-config ];
  buildInputs = [ curl.dev jansson criterion gtk4 ];
  installPhase = ''
    mkdir -p $out/bin
    install -m755 github_user_fetcher $out/bin/github_user_fetcher
    install -m755 github_user_fetcher_gui $out/bin/github_user_fetcher_gui
  '';
}
