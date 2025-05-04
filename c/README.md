# My C Project Template - GitHub User Fetcher

A complex C binary & library for fetching GitHub user data. This project includes external dependencies and tools and is packaged with **Flatpak** and **Docker**.

## Table of Contents
- [Installation](#installation)
- [Development Setup](#development-setup)
- [Commands](#commands)
  - [Build & Clean](#build--clean)
  - [Flatpak](#flatpak)
  - [Docker](#docker)
  - [Testing](#testing)
  - [Linting & Formatting](#linting--formatting)
- [Releases](#releases)
- [Documentation](#documentation)

## Installation

To install the release version of the project, use the following steps:

```bash
make release
make install
```

## Development Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/izelnakri/project-templates.git
   cd github-user-fetcher/c
   ```

2. **Install dependencies:**

   - Use `nix-shell` for the development environment or install manually. Don't forget to be on this nix shell, so you can have the required dependencies for the make commands.

   ```bash
   nix develop
   ```

## Commands

### Build & Clean

- **Build the project:**

  ```bash
  make build
  ```

- **Clean build artifacts:**

  ```bash
  make clean
  ```

- **Clean everything (including Flatpak and Docker):**

  ```bash
  make clean-all
  ```

- **Reset the project:**

  ```bash
  make reset
  ```

### Flatpak

- **Prepare and build the Flatpak package:**

  ```bash
  make flatpak-builder
  ```

- **Install the built Flatpak:**

  ```bash
  make install-flatpak
  ```

- **Run the Flatpak application:**

  ```bash
  make run-flatpak
  ```

- **Run the Flatpak CLI server application:**

  ```bash
  make run-flatpak-cli-server
  ```

### Docker

- **Build the Docker image:**

  ```bash
  make build-docker-image
  ```

- **Run the Docker CLI application:**

  ```bash
  make run-docker-cli
  ```

- **Run the Docker CLI application with a user:**

  ```bash
  make run-docker-cli-user
  ```

- **Run the Docker CLI server application:**

  ```bash
  make run-docker-cli-server
  ```

- **Run the Docker GUI application:**

  ```bash
  make run-docker-gui
  ```

### Testing

- **Run all tests:**

  ```bash
  make test
  ```

- **Run specific Clang-Tidy lints on source code:**

  ```bash
  make lint-clang-tidy
  ```

- **Run all benchmarks:**

  ```bash
  make bench
  ```

### Linting & Formatting

- **Check code format:**

  ```bash
  make format-check
  ```

- **Fix code format issues:**

  ```bash
  make format
  ```

- **Run static analysis tools (e.g., `clang-tidy`, `cppcheck`, `flawfinder`):**

  ```bash
  make lint
  ```

  - To run specific tools like `cppcheck` or `flawfinder`, use:

  ```bash
  make lint-cppcheck
  make lint-flawfinder
  ```

- **Automatically fix issues using `clang-tidy`:**

  ```bash
  make lint-fix
  ```

## Releases

### Build a Release

To create a release version of the project:

```bash
make release
```

This will build the project in **release** mode.

## Documentation

To generate and view the documentation:

```bash
make doc
```

This will generate the documentation and attempt to open it in **Brave**.

---

For further details, consult the [official documentation](https://izelnakri.github.io/project-templates/c) or refer to the [project repository](https://github.com/izelnakri/project-templates).
```
