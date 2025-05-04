{pkgs, unstable, self, system, ...}:
let
  # Helper function to get pkg-config paths (already defined in your flake.nix)
  getAllPkgConfigPaths = pkgs: inputs:
    let
      allDeps = pkgs.lib.closePropagation inputs;
    in
    pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "out" "share/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" allDeps;

  # Try to determine the current user
  currentUser = builtins.getEnv "USER";
  
  # Path to potential home-manager configuration
  userHomeConfig = "/home/${currentUser}/.config/home-manager/users/${currentUser}/default.nix";
  
  # Import your home-manager ZSH configuration if possible
  # Alternatively, we'll recreate the essential parts
  zshConfig = {
    enable = true;
    autosuggestion.enable = true;
    enableCompletion = true;
    defaultKeymap = "viins";
    syntaxHighlighting.enable = true;
    plugins = [{
      name = "vi-mode";
      src = unstable.zsh-vi-mode;
      file = "share/zsh-vi-mode/zsh-vi-mode.plugin.zsh";
    }];
  };

  # Function to generate zsh initialization script based on your config
  zshInitScript = ''
    # Load zsh plugins
    source ${unstable.zsh-vi-mode}/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh

    # Enable syntax highlighting
    source ${pkgs.zsh-syntax-highlighting}/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

    # Enable autosuggestions
    source ${pkgs.zsh-autosuggestions}/share/zsh-autosuggestions/zsh-autosuggestions.zsh

    # Set up completion
    autoload -U compinit && compinit

    # Vi mode settings
    bindkey -v
    export KEYTIMEOUT=1

    # Your custom functions from config
    unsetopt INC_APPEND_HISTORY
    setopt PROMPT_SUBST

    function parse_git_branch {
      git branch --no-color 2> /dev/null | sed -e '/^[^*]/d' -e 's/* \(.*\)/\ ->\ \1/'
    }

    autoload -U colors && colors

    # Prompt setup
    PROMPT='%{$fg[blue]%}$(date +%H:%M:%S) $(display_jobs_count_if_needed)%B%{$fg[green]%}%n %{$fg[blue]%}%~%{$fg[yellow]%}$(parse_git_branch) %{$reset_color%}';
    ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#808080,bold";

    # Edit line in vim with ctrl-e
    autoload edit-command-line; zle -N edit-command-line
    bindkey '^e' edit-command-line

    # Temporary zvm hacks for yanking & history search stuff
    zvm_vi_yank () {
      zvm_yank
      echo "$CUTBUFFER" | wl-copy -n
      zvm_exit_visual_mode
    }
    zvm_after_lazy_keybindings() {
      bindkey -M vicmd '^k' up-line-or-search
      bindkey -M vicmd '^j' down-line-or-search
    }
    zvm_after_init_commands+=("bindkey '^k' up-line-or-search" "bindkey '^j' down-line-or-search")

    # Additional function from your config
    edit() {
      nvim <($@ 2>&1)
    }

    # Project-specific aliases and functions
    alias run="make && ./github_user_fetcher"
    alias test="make test"
    alias format="make format"
    alias lint="make lint"
    alias bench="make bench"
    
    # Welcome message
    echo "🚀 GitHub User Fetcher Development Environment"
    echo "📋 Available commands:"
    echo "   - run: Build and run the project"
    echo "   - test: Run project tests"
    echo "   - format: Format code"
    echo "   - lint: Run linters"
    echo "   - bench: Run benchmarks"
    echo ""
    echo "🔧 Dev environment ready. Run: make"
  '';
in
pkgs.mkShell rec {
  inputsFrom = [ self.packages.${system}.default ];

  nativeBuildInputs = with pkgs; [
    pkg-config-unwrapped
    unstable.vcpkg
    appstream
    
    # Development tools
    clang-tools
    cppcheck
    flawfinder
    valgrind
    bear
    kcachegrind
    gdb
    flatpak-builder
    doxygen
    
    # Shell environment
    zsh
    zsh-syntax-highlighting
    zsh-autosuggestions
    unstable.zsh-vi-mode
    direnv
    
    # Other tools from your zshrc
    curl
    wl-clipboard
    neovim
    onefetch
    yazi
  ];
  
  buildInputs = [ self.packages.${system}.default ];

  PKG_CONFIG_PATH = getAllPkgConfigPaths pkgs (inputsFrom ++ buildInputs ++ nativeBuildInputs);

  # Set default shell to zsh with your configuration
  shellHook = with pkgs; ''
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:$HOME/.cache/vcpkg/packages/benchmark_x64-linux/lib/pkgconfig"
    export SHELL=${zsh}/bin/zsh
    export ZDOTDIR=$(mktemp -d)
    
    # Preserve original home directory reference
    export ORIG_HOME="$HOME"
    
    # Check if home-manager config exists and try to reference it
    if [ -f "$ORIG_HOME/.config/home-manager/users/$USER/default.nix" ]; then
      echo "Found home-manager config at $ORIG_HOME/.config/home-manager/users/$USER/default.nix"
      echo "Using zsh configuration from home-manager..."
      
      # Create a simplified zshrc that sources the original zshrc
      cat > $ZDOTDIR/.zshrc << EOF
      # Source original zsh configuration if available
      if [ -f "$ORIG_HOME/.zshrc" ]; then
        source "$ORIG_HOME/.zshrc"
      fi
      
      # Project-specific aliases and functions
      alias run="make && ./github_user_fetcher"
      alias test="make test"
      alias format="make format"
      alias lint="make lint"
      alias bench="make bench"
      
      # Welcome message
      echo "🚀 GitHub User Fetcher Development Environment"
      echo "📋 Available commands:"
      echo "   - run: Build and run the project"
      echo "   - test: Run project tests"
      echo "   - format: Format code"
      echo "   - lint: Run linters"
      echo "   - bench: Run benchmarks"
      echo ""
      echo "🔧 Dev environment ready. Run: make"
      EOF
    else
      echo "No home-manager config found, using bundled zsh configuration..."
      # Create temporary zshrc with your configuration
      cat > $ZDOTDIR/.zshrc << 'EOF'
      ${zshInitScript}
      EOF
    fi
    
    # Setup git hooks
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
    
    # Start zsh
    exec $SHELL
  '';
}
