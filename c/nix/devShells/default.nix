{pkgs, unstable, self, system, ...}:
let
  LS_COLORS = pkgs.fetchgit {
    url = "https://github.com/trapd00r/LS_COLORS";
    rev = "7d06cdb4a245640c3665fe312eb206ae758092be";
    sha256 = "ILT2tbJa6uOmxM0nzc4Vok8B6pF6MD1i+xJGgkehAuw=";
  };
  # Helper function to get pkg-config paths (already defined in your flake.nix)
  getAllPkgConfigPaths = pkgs: inputs:
    let
      allDeps = pkgs.lib.closePropagation inputs;
    in
    pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "out" "share/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" allDeps + ":" +
    pkgs.lib.makeSearchPathOutput "dev" "share/pkgconfig" allDeps;
  
  zshEntryScript = pkgs.writeShellScriptBin "enter-zsh-env" ''
    #!/usr/bin/env zsh
    export ZDOTDIR=$(mktemp -d)
    export ORIG_HOME="$HOME"
    
    cat > $ZDOTDIR/.zshrc << 'EOF'
if [ -f "$ORIG_HOME/.zshrc" ]; then
  echo "Using existing zsh configuration from $ORIG_HOME/.zshrc..."
  source "$ORIG_HOME/.zshrc"
else
  echo "Generating custom shell since no $ORIG_HOME/.zshrc found..."

  typeset -U path cdpath fpath manpath # Make these variables unique, thus faster command completion lookup:
  bindkey -v # Use viins keymap as the default.

  eval $(${pkgs.coreutils}/bin/dircolors ${LS_COLORS}/LS_COLORS)
  
  autoload -U colors && colors # Initialize colors
  autoload -U compinit && compinit # Initialize autocomplete system
  source ${pkgs.zsh-autosuggestions}/share/zsh-autosuggestions/zsh-autosuggestions.zsh
  ZSH_AUTOSUGGEST_STRATEGY=(history)
  ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#5E6E5E,bold";
  
  source ${unstable.zsh-vi-mode}/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh
  setopt SHARE_HISTORY # might be good between shells
  
  if [[ $options[zle] = on ]]; then
    . ${pkgs.skim}/share/skim/completion.zsh
    . ${pkgs.skim}/share/skim/key-bindings.zsh
  fi
  
  eval "$(${pkgs.direnv}/bin/direnv hook zsh)"
  
  autoload edit-command-line; zle -N edit-command-line
  bindkey '^e' edit-command-line # ctrl-e toggles edit on the last char of the line when insert?
  
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

  source ${pkgs.zsh-syntax-highlighting}/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
  ZSH_HIGHLIGHT_HIGHLIGHTERS+=(main)
fi

# Generate more DX-friendly prompt:
function display_jobs_count_if_needed {
  local job_count=$(jobs -s | wc -l | tr -d " ")

  if [ $job_count -gt 0 ]; then
    echo "%B%{$fg[yellow]%}|%j| ";
  fi
}

function parse_git_branch {
  git branch --no-color 2> /dev/null | sed -e '/^[^*]/d' -e 's/* \(.*\)/\->\ \1/'
}

export IS_NIX_SHELL=true;

setopt PROMPT_SUBST # Required -> Enable parameter expansion, command substitution and arithmetic expansion in the prompt.
PROMPT='%{$fg[blue]%}$(date +%H:%M:%S) $(display_jobs_count_if_needed)%B%{$fg[green]%}%n %{$fg[blue]%}%~%{$fg[cyan]%} ❄️%{$fg[yellow]%}$(parse_git_branch) %{$reset_color%}';
export PATH="$BIN_PATHS:$PATH"

# Utility functions & aliases:
function extract() {
    case "$1" in
        *.tar.bz|*.tar.bz2|*.tbz|*.tbz2) tar xjf "$1";;
        *.tar.gz|*.tgz) tar xzf "$1";;
        *.tar.xz|*.txz) tar xJf "$1";;
        *.zip) unzip "$1";;
        *.rar) unrar x "$1";;
        *.7z) 7z x "$1";;
    esac
}

function cheat {
  curl cheat.sh/$argv
}

alias run="make && ./github_user_fetcher"
alias test="make test"
alias format="make format"
alias lint="make lint"
alias bench="make bench"

echo "🚀 Dev environment ready. Run: make"
EOF

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
    
    exec zsh
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
  
  buildInputs = [ self.packages.${system}.default ];

  PKG_CONFIG_PATH = getAllPkgConfigPaths pkgs (inputsFrom ++ buildInputs ++ nativeBuildInputs);

  shellHook = with pkgs; ''
    export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:$HOME/.cache/vcpkg/packages/benchmark_x64-linux/lib/pkgconfig"
    export SHELL=${pkgs.zsh}/bin/zsh
    
    # Execute our ZSH script directly
    exec ${zshEntryScript}/bin/enter-zsh-env
  '';
}
