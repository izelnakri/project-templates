{ pkgs, unstable, LS_COLORS }:

pkgs.writeText "zshrc" ''
if [ -f "$ORIG_HOME/.zshrc" ]; then
  echo "Using existing zsh configuration from $ORIG_HOME/.zshrc..."
  source "$ORIG_HOME/.zshrc"
else
  echo "Generating custom shell since no $ORIG_HOME/.zshrc found..."

  # Basic ZSH configuration
  typeset -U path cdpath fpath manpath # Make these variables unique, thus faster command completion lookup
  bindkey -v # Use viins keymap as the default

  # Setup dircolors
  eval $(${pkgs.coreutils}/bin/dircolors ${LS_COLORS}/LS_COLORS)
  
  # Initialize autocomplete and colors
  autoload -U colors && colors
  autoload -U compinit && compinit
  
  # ZSH plugins
  source ${pkgs.zsh-autosuggestions}/share/zsh-autosuggestions/zsh-autosuggestions.zsh
  ZSH_AUTOSUGGEST_STRATEGY=(history)
  ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#5E6E5E,bold"
  
  source ${unstable.zsh-vi-mode}/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh
  
  # History settings
  setopt SHARE_HISTORY
  
  # Skim integration (fuzzy finder)
  if [[ $options[zle] = on ]]; then
    . ${pkgs.skim}/share/skim/completion.zsh
    . ${pkgs.skim}/share/skim/key-bindings.zsh
  fi
  
  # Direnv integration
  eval "$(${pkgs.direnv}/bin/direnv hook zsh)"
  
  # Command line editing
  autoload edit-command-line; zle -N edit-command-line
  bindkey '^e' edit-command-line # ctrl-e toggles edit on the last char of the line when insert?
  
  # Custom vi mode configurations
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

  # Syntax highlighting (load last for best results)
  source ${pkgs.zsh-syntax-highlighting}/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
  ZSH_HIGHLIGHT_HIGHLIGHTERS+=(main)
fi

# Prompt configuration functions
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

# Setup prompt
setopt PROMPT_SUBST # Required -> Enable parameter expansion, command substitution and arithmetic expansion in the prompt.
PROMPT='%{$fg[blue]%}$(date +%H:%M:%S) $(display_jobs_count_if_needed)%B%{$fg[green]%}%n %{$fg[blue]%}%~%{$fg[cyan]%} ❄️%{$fg[yellow]%}$(parse_git_branch) %{$reset_color%}';
export PATH="$BIN_PATHS:$PATH"

# Utility functions & aliases
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

# Project-specific aliases
alias run="make && ./github_user_fetcher"
alias test="make test"
alias format="make format"
alias lint="make lint"
alias bench="make bench"

# Greeting message
echo "🚀 Dev environment ready. Run: make"
''
