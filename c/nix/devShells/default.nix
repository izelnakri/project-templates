{pkgs, unstable, self, system, ...}:
let
  # External resources
  LS_COLORS = pkgs.fetchgit {
    url = "https://github.com/trapd00r/LS_COLORS";
    rev = "7d06cdb4a245640c3665fe312eb206ae758092be";
    sha256 = "ILT2tbJa6uOmxM0nzc4Vok8B6pF6MD1i+xJGgkehAuw=";
  };

  # Helper function for managing pkg-config paths
  getAllPkgConfigPaths = pkgs: inputs:
    let
      allDeps = pkgs.lib.closePropagation inputs;
    in
    pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "out" "share/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" allDeps;
  
  # ZSH configuration as a separate module
  zshConfig = import ./zsh-config.nix { 
    inherit pkgs unstable LS_COLORS; 
  };
  
  # Git hooks management script
  setupGitHooks = pkgs.writeShellScript "setup-git-hooks" ''
    if git rev-parse --git-dir > /dev/null 2>&1; then
      HOOKS_DIR="$(git rev-parse --git-dir)/hooks"
      PRE_COMMIT_HOOK_PATH="$HOOKS_DIR/pre-commit"

      FLAKE_DIR="$(dirname "$(readlink -f "$PWD/flake.nix")")"
      PRE_COMMIT_SOURCE="$FLAKE_DIR/pre-commit"
      
      mkdir -p "$HOOKS_DIR"

      if [ -f "$PRE_COMMIT_HOOK_PATH" ]; then
        rm "$PRE_COMMIT_HOOK_PATH"
        echo "🗑️ Removed existing pre-commit hook"
      fi

      if [ -f "$PRE_COMMIT_SOURCE" ]; then
        cp "$PRE_COMMIT_SOURCE" "$PRE_COMMIT_HOOK_PATH"
        chmod +x "$PRE_COMMIT_HOOK_PATH"
        echo "✅ Installed pre-commit hook from $PRE_COMMIT_SOURCE => $PRE_COMMIT_HOOK_PATH"
      else
        echo "❌ Could not find pre-commit script at $PRE_COMMIT_SOURCE"
      fi
    fi
  '';
  
  # Entry script that sets up the environment
  zshEntryScript = pkgs.writeShellScriptBin "enter-zsh-env" ''
    #!/usr/bin/env zsh
    export ZDOTDIR=$(mktemp -d)
    export ORIG_HOME="$HOME"
    
    # Copy our ZSH configuration to the temporary directory
    cp ${zshConfig} $ZDOTDIR/.zshrc
    
    # Setup git hooks
    ${setupGitHooks}
    
    # Enter the shell
    exec zsh
  '';

  # Grouped package lists for better organization
  devTools = with pkgs; [
    pkg-config-unwrapped
    unstable.vcpkg
    appstream
    clang-tools
    cppcheck
    flawfinder
    valgrind
    bear
    kcachegrind
    gdb
    flatpak-builder
    doxygen
  ];
  
  shellTools = with pkgs; [
    coreutils
    direnv
    skim
    zsh
    zsh-syntax-highlighting
    zsh-autosuggestions
    unstable.zsh-vi-mode
    neovim
    zshEntryScript
  ];
  
in
pkgs.mkShell rec {
  inputsFrom = [ self.packages.${system}.default ];

  # Split build inputs into logical groups
  nativeBuildInputs = devTools ++ shellTools;
  
  buildInputs = [ self.packages.${system}.default ];

  # Environment variables
  PKG_CONFIG_PATH = getAllPkgConfigPaths pkgs (inputsFrom ++ buildInputs ++ nativeBuildInputs);

  shellHook = with pkgs; ''
    set -eo pipefail;

    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:$HOME/.cache/vcpkg/packages/benchmark_x64-linux/lib/pkgconfig"
    
    if [ -z "$CI" ]; then
      export SHELL=${pkgs.zsh}/bin/zsh
      # Execute our ZSH script directly
      exec ${zshEntryScript}/bin/enter-zsh-env
    fi
    set -u # Fail undefined variables only on CI environment
  '';
}
