.DEFAULT_GOAL := all

BUILD_TYPE ?= debugoptimized
DEBUG ?= false
RELEASE ?= false

ifeq ($(DEBUG), true)
  BUILD_TYPE := debug
endif
ifeq ($(RELEASE), true)
  BUILD_TYPE := release
endif

clean-flatpak-build:
	rm -rf .flatpak-builder build flatpak/build flatpak/nix-environment-dependencies.tar.gz
prepare-flatpak-builder:
	sh flatpak/bundle-nix-environment-dependencies.sh
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

all: clean-all build

setup:
	meson setup build --buildtype=$(BUILD_TYPE)
build: clean setup
	ninja -C build
test:
	meson compile -C build && meson test -C build
run-cli: clean setup
	meson compile -C build && ./build/github_user_fetcher
run-cli-server: clean setup
	meson compile -C build && ./build/github_user_fetcher --server
run-server: run-cli-server
run-gui: clean setup
	meson compile -C build && ./build/github_user_fetcher_gui
cli:
	meson compile -C build && ./build/github_user_fetcher
cli-server:
	meson compile -C build && ./build/github_user_fetcher --server
server: cli-server
gui:
	meson compile -C build && ./build/github_user_fetcher_gui
clean:
	rm -rf build access.log
clean-all: clean clean-flatpak-build

develop:
	nix develop

build-docker-cli:
	nix build .#dockerImage
run-docker-cli: build-docker-cli
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher github_user_fetcher
run-docker-cli-user: build-docker-cli
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher github_user_fetcher --user izelnakri
run-docker-cli-server: build-docker-cli
	sudo docker load < ./result
	sudo docker run -it --rm \
		-p 1234:1234 \
		--init \
		github_user_fetcher github_user_fetcher --server
run-docker-gui: build-docker-cli
	sudo docker load < ./result
	sudo docker run -it --rm \
    --user $(shell id -u):$(shell id -g) \
		-e DISPLAY=$(DISPLAY) \
    -e DBUS_SESSION_BUS_ADDRESS=$(DBUS_SESSION_BUS_ADDRESS) \
		-v /etc/machine-id:/etc/machine-id:ro \
		-v /run/user/1000/bus:/run/user/1000/bus \
		-v /etc/fonts:/etc/fonts:ro \
		-v ~/.cache/fontconfig:/.cache/fontconfig \
		--device /dev/dri \
		--group-add $(shell getent group video | cut -d: -f3) \
		--ipc=host \
		--net=host \
		github_user_fetcher github_user_fetcher_gui

debug:
	@echo "CFLAGS: $(CFLAGS)"
	@echo "LDFLAGS: $(LDFLAGS)"
	@echo "LIBS: $(LIBS)"

bench: clean 
	meson setup build --buildtype=$(BUILD_TYPE) -Dbenchmarks=enabled
	meson compile -C build && meson test -C build --suite benchmark

bench-user:
	meson compile -C build && ./build/bench_user

release:
	@echo "Building in RELEASE mode"
	make RELEASE=true setup
	make RELEASE=true build

install: release
	@echo "Installing release build"
	sudo meson install -C build

uninstall:
	@echo "Removing installed release build"
	sudo ninja uninstall -C build
