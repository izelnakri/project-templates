clean-flatpak-build:
	rm -rf .flatpak-builder build flatpak/build flatpak/nix-deps.tar.gz
prepare-flatpak-builder:
	sh flatpak/prepare-nix-deps.sh
flatpak-builder: 
	flatpak-builder flatpak/build flatpak/org.example.GithubUserFetcher.json --force-clean
build-flatpak: clean-flatpak-build prepare-flatpak-builder flatpak-builder
run-flatpak-cli: build-flatpak
	echo "Running org.example.GithubUserFetcher.json CLI:"
	flatpak-builder --run flatpak/build flatpak/org.example.GithubUserFetcher.json github_user_fetcher
run-flatpak-cli-server: build-flatpak
	echo "Running org.example.GithubUserFetcher.json CLI server:"
	flatpak-builder --run flatpak/build flatpak/org.example.GithubUserFetcher.json github_user_fetcher --server
run-flatpak-gui: build-flatpak
	echo "Running org.example.GithubUserFetcher.json GUI:"
	flatpak-builder --run flatpak/build flatpak/org.example.GithubUserFetcher.json github_user_fetcher_gui
run-flatpak: run-flatpak-gui

setup:
	meson setup build
build: clean setup
	ninja -C build
test:
	meson compile -C build && meson test -C build
run-cli:
	meson compile -C build && ./build/github_user_fetcher
run-server:
	meson compile -C build && ./build/github_user_fetcher --server
run-gui:
	meson compile -C build && ./build/github_user_fetcher_gui
clean:
	rm -rf build

debug:
	@echo "CFLAGS: $(CFLAGS)"
	@echo "LDFLAGS: $(LDFLAGS)"
	@echo "LIBS: $(LIBS)"
