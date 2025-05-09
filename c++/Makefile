# TODO: Adjust it for cpp+ further
# TODO: run & refactor $ make format
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

# INCLUDE_DIRS = -I./src -I./vendor/civetweb # NOTE: This was previously
INCLUDE_DIRS = -I./src

C_SOURCES := $(wildcard src/*.c)
CPP_SOURCES := $(wildcard src/*.cpp benchmarks/*.cpp)
H_SOURCES := $(wildcard src/*.h)
HPP_SOURCES := $(wildcard src/*.hpp benchmarks/*.hpp)
ALL_SOURCES := $(C_SOURCES) $(CPP_SOURCES) $(H_SOURCES) $(HPP_SOURCES)

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
	echo "Running manifest.json CLI:"
	flatpak-builder --run flatpak/build flatpak/manifest.json github_user_fetcher
flatpak-run-cli-server: flatpak-build
	echo "Running manifest.json CLI server:"
	flatpak-builder --run flatpak/build flatpak/manifest.json github_user_fetcher --server
flatpak-run-gui: flatpak-build
	echo "Running manifest.json GUI:"
	flatpak-builder --run flatpak/build flatpak/manifest.json github_user_fetcher_gui
run-flatpak: flatpak-run-gui

all: clean-all build

setup:
	meson setup build --buildtype=$(BUILD_TYPE)
check-setup:
	@if [ ! -f build/meson-info/intro-dependencies.json ]; then \
		echo "Run 'meson setup build' first!"; \
		make setup; \
	fi
	
build: clean setup
	ninja -C build
clean:
	rm -rf build access.log
clean-all: clean flatpak-clean
	rm -rf docs/html
reset: clean-all build

develop:
	@if [ -z "$$IS_NIX_SHELL" ]; then nix develop; else echo "Already in nix shell"; fi
test: check-setup
	meson compile -C build && meson test -C build
run-cli: check-setup
	meson compile -C build && ./build/github_user_fetcher
run-cli-server: check-setup
	meson compile -C build && ./build/github_user_fetcher --server
run-server: run-cli-server
run-gui: clean setup
	meson compile -C build && ./build/github_user_fetcher_gui
cli: check-setup
	meson compile -C build && ./build/github_user_fetcher
cli-server: check-setup
	meson compile -C build && ./build/github_user_fetcher --server
server: cli-server
gui: check-setup
	meson compile -C build && ./build/github_user_fetcher_gui

compile-commands:
	@if [ ! -d build ]; then meson setup build --buildtype=$(BUILD_TYPE); fi
	cd build && ninja -t compdb > compile_commands.json
	cp build/compile_commands.json .

lint: lint-clang-tidy lint-cppcheck lint-flawfinder
lint-clang-tidy: compile-commands
	@echo "Running clang-tidy..."
	@find src -name "*.c" -o -name "*.h" | xargs -r clang-tidy -p .
lint-cppcheck:
	@echo "Running cppcheck..."
	cppcheck --enable=all --error-exitcode=1 --suppress=unusedStructMember --suppress=missingIncludeSystem --inline-suppr --quiet --check-level=exhaustive $(INCLUDE_DIRS) $(ALL_SOURCES)
lint-flawfinder:
	@echo "Running flawfinder (security check)..."
	flawfinder --minlevel=1 $(ALL_SOURCES)
lint-fix:
	@find src -name "*.c" -o -name "*.h" | xargs -r clang-tidy -fix -fix-errors -p .

format:
	@echo "Formatting code with clang-format..."
	@find src tests benchmarks -name "*.c" -o -name "*.h" -o -name "*.cpp" | xargs clang-format -i -style=file
	@echo "Code formatting complete."
format-check:
	@echo "Checking code formatting..."
	@find src tests benchmarks -name "*.c" -o -name "*.h" -o -name "*.cpp" | xargs -I{} bash -c 'clang-format -style=file {} | diff --color=always -u {} - || printf "\033[31m=> File {} needs formatting\033[0m\n"'

docker-build-image:
ifeq ($(RELEASE), true)
	nix build .#dockerProductionImage
else
	nix build .#dockerImage
endif
docker-run-cli: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher github_user_fetcher
docker-run-cli-user: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher github_user_fetcher --user izelnakri
docker-run-cli-server: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm \
		-p 1234:1234 \
		--init \
		github_user_fetcher github_user_fetcher --server
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
		github_user_fetcher github_user_fetcher_gui
docker-debug: docker-build-image
	sudo docker load < ./result
	sudo docker run -it --rm github_user_fetcher zsh

debug:
	@echo "CFLAGS: $(CFLAGS)"
	@echo "LDFLAGS: $(LDFLAGS)"
	@echo "LIBS: $(LIBS)"

doc:
	@echo "Generating documentation..."
	@chmod +x docs/build.sh
	@./docs/build.sh
	@echo "Opening documentation in Brave..."
	@brave docs/html/index.html || echo "Failed to open documentation in Brave"

bench: clean 
	meson setup build --buildtype=$(BUILD_TYPE) -Dbenchmarks=enabled
	meson compile -C build && meson test -C build --suite benchmark

bench-user:
	meson compile -C build && ./build/bench_user

valgrind-cli: build
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose ./build/github_user_fetcher

valgrind-cli-server: build
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose ./build/github_user_fetcher --server

valgrind-gui: build
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose ./build/github_user_fetcher_gui

valgrind-callgrind: build
	valgrind --tool=callgrind ./build/github_user_fetcher
	@echo "Use 'kcachegrind callgrind.out.*' to visualize the call graph"

valgrind-cachegrind: build
	valgrind --tool=cachegrind ./build/github_user_fetcher
	@echo "Use 'kcachegrind cachegrind.out.*' to visualize the cache profile"

valgrind-massif: build
	valgrind --tool=massif ./build/github_user_fetcher
	@echo "Use 'ms_print massif.out.*' to view heap profile"

release:
	@echo "Building in RELEASE mode"
	make RELEASE=true setup
	make RELEASE=true build

install: release
	$(if $(PREFIX), meson configure build --prefix=$(PREFIX))
	@echo "Installing release build"
	sudo meson install -C build

uninstall:
	@echo "Removing installed release build"
	sudo ninja uninstall -C build
