#!/usr/bin/env bash
set -e

# --- Configuration ---
DOXYGEN_TIMEOUT_SECONDS="1800" # 30 minutes. Adjust, or set to "" to disable.
ENABLE_DIAGRAMS="YES"          # Set to "YES" to enable dot/class diagrams (slower, needs Graphviz)
MAX_HEADER_FIND_DEPTH_FOR_DEPS="1" # Max depth for 'find' in dependency include dirs. 1=top-level, 2=one subdir.

CACHE_FILE="build/docs/.cache"
DOXYFILE_BASE="docs/Doxyfile" # Your base Doxyfile (can be minimal, script overrides many settings)
BUILD_DIR="build"
DOCS_DIR="$BUILD_DIR/docs"
HTML_OUTPUT="docs"
PROJECT_ROOT=$(pwd)

# --- Create necessary directories ---
mkdir -p "$BUILD_DIR" "$DOCS_DIR" "$PROJECT_ROOT/$HTML_OUTPUT"

# --- Meson Build Directory Setup ---
if [ ! -f "$BUILD_DIR/compile_commands.json" ]; then
  echo "Setting up build directory (compile_commands.json not found)..."
  if [ -f "$BUILD_DIR/build.ninja" ]; then
      echo "Build directory exists, attempting to reconfigure to generate compile_commands.json..."
      meson configure "$BUILD_DIR" > /dev/null # Suppress verbose output unless error
  else
      meson setup "$BUILD_DIR" > /dev/null
  fi
  if [ ! -f "$BUILD_DIR/compile_commands.json" ]; then
      echo "Retrying meson setup with --wipe..."
      meson setup "$BUILD_DIR" --wipe || meson setup "$BUILD_DIR" > /dev/null
  fi
fi
if [ ! -f "$BUILD_DIR/compile_commands.json" ]; then
    echo "ERROR: compile_commands.json could not be found or generated in '$BUILD_DIR'."
    exit 1
fi

# --- Function to check if rebuild is needed ---
needs_rebuild() {
  if [ ! -f "$CACHE_FILE" ]; then echo "Cache file '$CACHE_FILE' not found. Rebuilding." && return 0; fi
  if [ -n "$(find "$PROJECT_ROOT/src" "$PROJECT_ROOT/include" -path "$PROJECT_ROOT/src" -prune -o -path "$PROJECT_ROOT/include" -prune -o -type f \( -name "*.cpp" -o -name "*.h" -o -name "*.hpp" -o -name "*.cc" -o -name "*.hxx" \) -newer "$CACHE_FILE" -print -quit 2>/dev/null)" ]; then
    echo "Project source/include files changed. Rebuilding." && return 0
  fi
  if [ -f "$DOXYFILE_BASE" ] && [ "$DOXYFILE_BASE" -nt "$CACHE_FILE" ]; then echo "Base Doxyfile '$DOXYFILE_BASE' changed. Rebuilding." && return 0; fi
  if [ "$0" -nt "$CACHE_FILE" ]; then echo "This script '$0' changed. Rebuilding." && return 0; fi
  if [ "$BUILD_DIR/compile_commands.json" -nt "$CACHE_FILE" ]; then echo "'$BUILD_DIR/compile_commands.json' changed. Rebuilding." && return 0; fi
  echo "No significant changes detected. Documentation appears up to date."
  return 1
}

if needs_rebuild; then
  echo "Changes detected or rebuild forced, rebuilding documentation..."
  echo "#######################################################################################"
  echo "## WARNING: Doxygen will parse project code + headers of ALL included dependencies.  ##"
  echo "## This configuration prioritizes speed by limiting depth in dependency headers.     ##"
  echo "## Processing can still be lengthy for large projects. Check logs for issues.        ##"
  echo "#######################################################################################"
  sleep 3

  # --- Prepare Doxygen INPUT and INCLUDE_PATH lists ---
  echo "Preparing Doxygen input and include path lists..."
  DOXYGEN_INPUT_LIST_FILE="$DOCS_DIR/doxygen_inputs_list.txt"
  DOXYGEN_INCLUDE_PATH_LIST_FILE="$DOCS_DIR/doxygen_include_paths_list.txt"
  > "$DOXYGEN_INPUT_LIST_FILE"
  > "$DOXYGEN_INCLUDE_PATH_LIST_FILE"

  # 1. Project's own source code directories
  if [ -d "$PROJECT_ROOT/src" ]; then
    echo "$PROJECT_ROOT/src" >> "$DOXYGEN_INPUT_LIST_FILE"
    echo "$PROJECT_ROOT/src" >> "$DOXYGEN_INCLUDE_PATH_LIST_FILE"
  else
    echo "INFO: Project 'src' directory not found at '$PROJECT_ROOT/src'."
  fi
  if [ -d "$PROJECT_ROOT/include" ]; then
    echo "$PROJECT_ROOT/include" >> "$DOXYGEN_INPUT_LIST_FILE"
    echo "$PROJECT_ROOT/include" >> "$DOXYGEN_INCLUDE_PATH_LIST_FILE"
  else
    echo "INFO: Project 'include' directory not found at '$PROJECT_ROOT/include'."
  fi
  if [ -f "$PROJECT_ROOT/README.md" ]; then
    echo "$PROJECT_ROOT/README.md" >> "$DOXYGEN_INPUT_LIST_FILE"
  fi

  # 2. Process external include paths from compile_commands.json
  if [ -f "$BUILD_DIR/compile_commands.json" ]; then
    echo "Processing compile_commands.json for dependency headers and include paths..."
    grep -o -- "-I[^ \"]*" "$BUILD_DIR/compile_commands.json" |
      sed 's/^-I//g' |
      grep -Ev "^(/run|/proc|/sys|/var|/tmp|/dev|/etc)(/|$)" |
      grep -Ev "^/usr/include/systemd(/|$)" |
      sort -u > "$DOCS_DIR/all_discovered_include_dirs.txt"

    echo "Identifying headers from dependencies (maxdepth $MAX_HEADER_FIND_DEPTH_FOR_DEPS) for INPUT..."
    echo "Populating INCLUDE_PATH with dependency directories..."
    while IFS= read -r path_from_compile_commands; do
      resolved_path="$path_from_compile_commands"
      if ! [[ "$resolved_path" = /* ]]; then
        if [ -d "$PROJECT_ROOT/$resolved_path" ]; then resolved_path="$PROJECT_ROOT/$resolved_path";
        elif [ -d "$BUILD_DIR/$resolved_path" ]; then resolved_path="$BUILD_DIR/$resolved_path";
        fi
      fi

      if [ -d "$resolved_path" ] && [ -r "$resolved_path" ]; then
          canonical_path_dir=$(realpath -m "$resolved_path")
          echo "$canonical_path_dir" >> "$DOXYGEN_INCLUDE_PATH_LIST_FILE"
          find "$canonical_path_dir" -maxdepth "$MAX_HEADER_FIND_DEPTH_FOR_DEPS" -type f \( -name "*.h" -o -name "*.hpp" -o -name "*.hxx" \) >> "$DOXYGEN_INPUT_LIST_FILE" 2>/dev/null
      fi
    done < "$DOCS_DIR/all_discovered_include_dirs.txt"
  fi

  doxy_final_inputs=$(sort -u "$DOXYGEN_INPUT_LIST_FILE" | grep -v '^$' | tr '\n' ' ')
  doxy_final_include_paths=$(sort -u "$DOXYGEN_INCLUDE_PATH_LIST_FILE" | grep -v '^$' | tr '\n' ' ')

  if [ -z "$doxy_final_inputs" ]; then echo "ERROR: No input paths for Doxygen. Aborting." && exit 1; fi
  # It's possible for include paths to be empty if only project files are used and no external includes, but less likely
  # if [ -z "$doxy_final_include_paths" ]; then echo "ERROR: No include paths for Doxygen. Aborting." && exit 1; fi
  echo "Doxygen input and include paths prepared."

  # --- Doxyfile Generation ---
  echo "Generating temporary Doxyfile ($DOCS_DIR/Doxyfile.tmp)..."
  DOXYFILE_TMP="$DOCS_DIR/Doxyfile.tmp"
  if [ -f "$DOXYFILE_BASE" ]; then cp "$DOXYFILE_BASE" "$DOXYFILE_TMP"; else echo "# Auto-generated Doxyfile by build.sh" > "$DOXYFILE_TMP"; fi

  # Remove settings that will be explicitly set by this script
  sed -i -e '/^PROJECT_NAME[[:space:]]*=/d' \
         -e '/^OUTPUT_DIRECTORY[[:space:]]*=/d' \
         -e '/^INPUT[[:space:]]*=/d' \
         -e '/^INCLUDE_PATH[[:space:]]*=/d' \
         -e '/^USE_MDFILE_AS_MAINPAGE[[:space:]]*=/d' \
         -e '/^RECURSIVE[[:space:]]*=/d' \
         -e '/^FILE_PATTERNS[[:space:]]*=/d' \
         -e '/^EXCLUDE_PATTERNS[[:space:]]*=/d' \
         -e '/^HAVE_DOT[[:space:]]*=/d' \
         -e '/^CLASS_DIAGRAMS[[:space:]]*=/d' \
         -e '/^EXTRACT_STATIC[[:space:]]*=/d' \
         -e '/^EXTRACT_LOCAL_CLASSES[[:space:]]*=/d' \
         -e '/^SOURCE_BROWSER[[:space:]]*=/d' \
         -e '/^WARN_LOGFILE[[:space:]]*=/d' \
         "$DOXYFILE_TMP"

  HAVE_DOT_SETTING="NO"
  if [[ "$ENABLE_DIAGRAMS" == "YES" ]]; then
    if command -v dot &> /dev/null; then
      HAVE_DOT_SETTING="YES"; echo "Graphviz 'dot' found and diagrams enabled by script."
    else
      echo "ENABLE_DIAGRAMS=YES but 'dot' command not found. Diagrams disabled by script."
    fi
  else
    echo "Diagram generation disabled by ENABLE_DIAGRAMS script setting."
  fi

  cat >> "$DOXYFILE_TMP" << EOF

# --- Settings Managed by build.sh ---
PROJECT_NAME           = "Project Code Navigation (Optimized)"
OUTPUT_DIRECTORY       = "$PROJECT_ROOT/$HTML_OUTPUT"
INPUT                  = $doxy_final_inputs
INCLUDE_PATH           = $doxy_final_include_paths
USE_MDFILE_AS_MAINPAGE = $([ -f "$PROJECT_ROOT/README.md" ] && echo "$PROJECT_ROOT/README.md" || echo "")
RECURSIVE              = YES
FILE_PATTERNS          = *.c *.cc *.cpp *.cxx *.h *.hh *.hpp *.hxx
SOURCE_BROWSER         = YES
BUILTIN_STL_SUPPORT    = YES
HAVE_DOT               = $HAVE_DOT_SETTING
# CLASS_DIAGRAMS is obsolete, use HAVE_DOT and graph specific tags like CLASS_GRAPH (default YES if HAVE_DOT=YES)
MAX_DOT_GRAPH_DEPTH    = 1 # Keep low for speed if diagrams are enabled
EXTRACT_STATIC         = NO # For speed and stability
EXTRACT_LOCAL_CLASSES  = NO # For speed and stability
WARN_LOGFILE           = "$PROJECT_ROOT/$DOCS_DIR/doxygen_warnings.log"
QUIET                  = YES
WARNINGS               = YES
WARN_IF_DOC_ERROR      = YES
WARN_IF_UNDOCUMENTED   = NO
EXCLUDE_PATTERNS       = */test/* */tests/* */example/* */examples/* \\
                         */build/* */build-*/* \\
                         */\.git/* */\.svn/* */CMakeFiles/* \\
                         */\.vscode/* */\.idea/* \\
                         */.*/private/* */.*/Private/* */.*/detail/* */.*_detail/* \\
                         *moc_* qrc_* ui_*
# Consider adding more EXCLUDE_PATTERNS for specific large/problematic library internals
# e.g., */boost/asio/impl/* */gtkmm/private/*
# --- End of build.sh Managed Settings ---
EOF
  echo "Temporary Doxyfile generated."

  # --- Execute Doxygen ---
  echo "Starting Doxygen build. This may take some time but is optimized..."
  export DOXYGEN_BUFFER_SIZE="131072"

  cmd_parts=()
  if [ -n "$DOXYGEN_TIMEOUT_SECONDS" ] && command -v timeout &> /dev/null; then
    cmd_parts+=("timeout" "${DOXYGEN_TIMEOUT_SECONDS}s")
  fi
  cmd_parts+=("doxygen" "$DOXYFILE_TMP")

  echo "Executing: ${cmd_parts[*]}"
  if ! "${cmd_parts[@]}"; then
      echo "ERROR: Doxygen command failed or was terminated. Review output and log."
      echo "Log file: $PROJECT_ROOT/$DOCS_DIR/doxygen_warnings.log"
      # Check for specific exit codes, e.g., timeout (124) vs segfault (139)
      # This script uses 'set -e', so it would exit here. This is more for clarity.
      exit 1
  fi

  touch "$CACHE_FILE"
  echo "#######################################################################################"
  echo "## Doxygen processing finished. Output: $PROJECT_ROOT/$HTML_OUTPUT                     ##"
  echo "## Review log for warnings/errors: $PROJECT_ROOT/$DOCS_DIR/doxygen_warnings.log                            ##"
  echo "#######################################################################################"
else
  echo "No significant changes detected. Documentation is up to date."
fi
