.DEFAULT_GOAL := all

BUILD_TYPE ?= Debug
CMAKE_FLAGS ?=
PARALLEL_JOBS = $(shell nproc || echo 2)

ifeq ($(DEBUG), true)
  BUILD_TYPE := Debug
endif
ifeq ($(RELEASE), true)
  BUILD_TYPE := Release
endif

all: clean-all build

setup:
	cmake -B build -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) $(CMAKE_FLAGS)

check-setup:
	@if [ ! -d build ]; then \
		echo "Run 'cmake -B build' first!"; \
    make -f Makefile.cmake setup; \
	fi

build: check-setup
	cmake --build build -j$(PARALLEL_JOBS)

clean:
	rm -rf build access.log

clean-all: clean flatpak-clean
	rm -rf docs/html

reset: clean-all build

develop:
	@if [ -z "$$IS_NIX_SHELL" ]; then nix develop; else echo "Already in nix shell"; fi

test: check-setup
	cmake --build build -j$(PARALLEL_JOBS) && cd build && ctest --output-on-failure

test-unit: check-setup
	cmake --build build --target test_unit -j$(PARALLEL_JOBS) && cd build && ctest -L unit --output-on-failure

test-api: check-setup
	cmake --build build --target test_api -j$(PARALLEL_JOBS) && cd build && ctest -L api --output-on-failure

test-cli: check-setup
	cmake --build build -j$(PARALLEL_JOBS) && cd build && ctest -L cli --output-on-failure

run-cli: check-setup
	cmake --build build -j$(PARALLEL_JOBS) && ./build/github_user_fetcher

run-cli-server: check-setup
	cmake --build build -j$(PARALLEL_JOBS) && ./build/github_user_fetcher --server

run-server: run-cli-server

run-gui: check-setup
	cmake --build build --target github_user_fetcher_gui -j$(PARALLEL_JOBS) && ./build/github_user_fetcher_gui

cli: run-cli

cli-server: run-cli-server

server: cli-server

gui: run-gui

compile-commands:
	@if [ ! -d build ]; then cmake -B build -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) -DCMAKE_EXPORT_COMPILE_COMMANDS=ON; else cmake -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON; fi
	cp build/compile_commands.json .

lint: lint-clang-tidy lint-cppcheck lint-flawfinder

lint-clang-tidy: compile-commands
	@echo "Running clang-tidy..."
	@find src -name "*.c" -o -name "*.cpp" -o -name "*.h" -o -name "*.hpp" | xargs -r clang-tidy -p .

lint-cppcheck:
	@echo "Running cppcheck..."
	cppcheck --enable=all --error-exitcode=1 --suppress=unusedStructMember --suppress=missingIncludeSystem --inline-suppr --quiet --check-level=exhaustive -I./src $(wildcard src/*.c src/*.cpp src/*.h src/*.hpp)

lint-flawfinder:
	@echo "Running flawfinder (security check)..."
	flawfinder --minlevel=1 $(wildcard src/*.c src/*.cpp src/*.h src/*.hpp benchmarks/*.cpp benchmarks/*.hpp)

lint-fix:
	@find src -name "*.c" -o -name "*.cpp" -o -name "*.h" -o -name "*.hpp" | xargs -r clang-tidy -fix -fix-errors -p .

format:
	@echo "Formatting code with clang-format..."
	@find src tests benchmarks -name "*.c" -o -name "*.h" -o -name "*.cpp" -o -name "*.hpp" | xargs clang-format -i -style=file
	@echo "Code formatting complete."

format-check:
	@echo "Checking code formatting..."
	@find src tests benchmarks -name "*.c" -o -name "*.h" -o -name "*.cpp" -o -name "*.hpp" | xargs -I{} bash -c 'clang-format -style=file {} | diff --color=always -u {} - || printf "\033[31m=> File {} needs formatting\033[0m\n"'

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
	@echo "CMAKE_FLAGS: $(CMAKE_FLAGS)"
	@echo "BUILD_TYPE: $(BUILD_TYPE)"

doc:
	@echo "Generating documentation..."
	@chmod +x docs/build.sh
	@./docs/build.sh
	@echo "Opening documentation in Brave..."
	@brave docs/html/index.html || echo "Failed to open documentation in Brave"

bench: clean
	cmake -B build -DCMAKE_BUILD_TYPE=$(BUILD_TYPE) -DBUILD_BENCHMARKS=ON
	cmake --build build --target benchmarks -j$(PARALLEL_JOBS)
	cd build && ctest -L benchmark

bench-user:
	cmake --build build -j$(PARALLEL_JOBS) && ./build/benchmarks

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

heaptrack: build
	heaptrack ./build/github_user_fetcher --user izelnakri

release:
	@echo "Building in RELEASE mode"
	make BUILD_TYPE=Release -f Makefile.cmake setup
	make BUILD_TYPE=Release -f Makefile.cmake build

install: release
	@echo "Installing release build"
	cd build && sudo cmake --install .

uninstall:
	@echo "Removing installed release build"
	@if [ -f build/install_manifest.txt ]; then \
		sudo xargs rm -f < build/install_manifest.txt; \
	else \
		echo "No install manifest found. Cannot uninstall."; \
	fi

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
