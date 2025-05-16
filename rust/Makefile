.DEFAULT_GOAL := all

RELEASE ?= false
DEBUG ?= false

ifeq ($(RELEASE), true)
  BUILD_MODE := --release
  TARGET_DIR := target/release
else
  BUILD_MODE :=
  TARGET_DIR := target/debug
endif

# Binaries
CLI_BIN := github_user_fetcher
GUI_BIN := github_user_fetcher_gui

# Rust sources (used by format/lint)
# RS_SOURCES := $(shell find src benches tests examples -name '*.rs')
RS_SOURCES := $(shell find src tests -name '*.rs')

all: clean-all build

clean:
	cargo clean

clean-all: clean flatpak-clean
	rm -rf docs/html

reset: clean-all build

setup:
	cargo fetch

check-setup:
	@echo "Cargo handles setup automatically"

build: setup
	cargo build $(BUILD_MODE)

develop:
	@if [ -z "$$IN_NIX_SHELL" ]; then nix develop; else echo "Already in nix shell"; fi

test:
	cargo test --all

test-unit:
	cargo test --lib

test-api:
	cargo test --test api

test-cli:
	cargo test --test cli

run-cli: build
	$(TARGET_DIR)/$(CLI_BIN)

run-cli-server: build
	$(TARGET_DIR)/$(CLI_BIN) --server

run-server: run-cli-server

run-gui: build
	$(TARGET_DIR)/$(GUI_BIN)

cli: run-cli
cli-server: run-cli-server
server: run-server
gui: run-gui

format:
	cargo fmt

format-check:
	cargo fmt -- --check

lint: lint-clippy
lint-clippy:
	cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged

compile-commands:
	cargo install cargo-compdb --locked || true
	cargo compdb > compile_commands.json

doc:
	cargo doc --no-deps --document-private-items
	@echo "Opening documentation in Brave..."
	@brave target/doc/index.html || echo "Failed to open documentation in Brave"
	# rather it was target/doc/$(BIN_NAME)/index.html # TODO: maybe remove --no-deps

# Flatpak
flatpak-clean:
	rm -rf .flatpak-builder build flatpak/build flatpak/nix-environment-dependencies.tar.gz

flatpak-prepare-build:
	sh flatpak/bundle-nix-environment-dependencies.sh

flatpak-builder:
	flatpak-builder flatpak/build flatpak/manifest.json --force-clean

flatpak-build: flatpak-clean flatpak-prepare-build flatpak-builder

flatpak-install:
	flatpak build-export local-flatpak-repo flatpak/build
	flatpak remote-add --user --no-gpg-verify local-last-exported-flatpak-remote ./local-flatpak-repo
	flatpak install --user local-last-exported-flatpak-remote org.example.GithubUserFetcher

flatpak-run-cli: flatpak-build
	flatpak-builder --run flatpak/build flatpak/manifest.json $(CLI_BIN)

flatpak-run-cli-server: flatpak-build
	flatpak-builder --run flatpak/build flatpak/manifest.json $(CLI_BIN) --server

flatpak-run-gui: flatpak-build
	flatpak-builder --run flatpak/build flatpak/manifest.json $(GUI_BIN)

run-flatpak: flatpak-run-gui

# Docker
docker-build-image:
ifeq ($(RELEASE), true)
	nix build .#dockerProductionImage
else
	nix build .#dockerImage
endif

docker-run-cli: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher $(CLI_BIN)

docker-run-cli-user: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher $(CLI_BIN) --user izelnakri

docker-run-cli-server: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm -p 1234:1234 --init github_user_fetcher $(CLI_BIN) --server

docker-run-gui: docker-build-image
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
		github_user_fetcher $(GUI_BIN)

docker-debug: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher zsh

release:
	@echo "Building in RELEASE mode"
	$(MAKE) RELEASE=true build

install: release
	cargo install --path . --locked $(if $(PREFIX), --root $(PREFIX),)

uninstall:
	cargo uninstall github_user_fetcher || echo "Nothing to uninstall"

# Valgrind/Heaptrack (use with C-compiled Rust libs or via `cargo valgrind`)
valgrind-cli: build
	valgrind --leak-check=full --track-origins=yes $(TARGET_DIR)/$(CLI_BIN)

valgrind-cli-server: build
	valgrind --leak-check=full --track-origins=yes $(TARGET_DIR)/$(CLI_BIN) --server

valgrind-gui: build
	valgrind --leak-check=full --track-origins=yes $(TARGET_DIR)/$(GUI_BIN)

valgrind-callgrind: build
	valgrind --tool=callgrind $(TARGET_DIR)/$(CLI_BIN)
	@echo "Use 'kcachegrind callgrind.out.*' to visualize the call graph"

valgrind-cachegrind: build
	valgrind --tool=cachegrind $(TARGET_DIR)/$(CLI_BIN)
	@echo "Use 'kcachegrind cachegrind.out.*' to visualize the cache profile"

valgrind-massif: build
	valgrind --tool=massif $(TARGET_DIR)/$(CLI_BIN)
	@echo "Use 'ms_print massif.out.*' to view heap profile"

heaptrack: build
	heaptrack $(TARGET_DIR)/$(CLI_BIN) --user izelnakri
