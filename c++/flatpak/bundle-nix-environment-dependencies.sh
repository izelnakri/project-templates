#!/usr/bin/env bash
set -euo pipefail

# Define packages - only what's strictly necessary
INITIAL_PACKAGES=("boost" "nlohmann_json" "gtkmm4")

TEMP_DIR=$(mktemp -d)
FINAL_STAGE="$TEMP_DIR/final"
PC_TEMP="$TEMP_DIR/pcfiles"

mkdir -p "$FINAL_STAGE/include" "$FINAL_STAGE/lib/pkgconfig" "$FINAL_STAGE/bin" "$FINAL_STAGE/share" "$PC_TEMP"

declare -A VISITED
ALL_PACKAGES=("${INITIAL_PACKAGES[@]}")

# This allows for parallel execution of commands
MAX_JOBS=$(nproc)
echo "🚀 Using $MAX_JOBS parallel jobs for faster processing"

function resolve_nix_path() {
  nix eval --inputs-from . --raw "nixpkgs#$1" 2>/dev/null || true
}

function fetch_pc_files() {
  local pkg="$1"
  for try_pkg in "$pkg.dev" "$pkg"; do
    local dev_path
    dev_path=$(resolve_nix_path "$try_pkg")
    if [[ -n "$dev_path" && -d "$dev_path/lib/pkgconfig" ]]; then
      cp -Lr "$dev_path/lib/pkgconfig/"*.pc "$PC_TEMP/" 2>/dev/null || true
      break  # Exit after finding the first matching package
    fi
  done
}

function parse_pc_requires() {
  grep -h -E '^Requires(\.private)?\s*:' "$PC_TEMP"/*.pc 2>/dev/null \
    | cut -d: -f2- \
    | tr ',' '\n' \
    | awk '{print $1}' \
    | sed 's/[><=].*//' \
    | grep -v '^$' \
    | sort -u || echo ""
}

function pc_file_exists() {
  [[ -f "$FINAL_STAGE/lib/pkgconfig/$1.pc" ]] || [[ -f "$PC_TEMP/$1.pc" ]]
}

# STEP 1: Fetch initial PC files (in parallel)
echo "📦 Fetching initial package-config files..."
pids=()
for pkg in "${INITIAL_PACKAGES[@]}"; do
  {
    fetch_pc_files "$pkg"
    VISITED[$pkg]=1
  } &
  pids+=($!)
  
  # Control number of parallel jobs
  if [[ ${#pids[@]} -ge $MAX_JOBS ]]; then
    wait "${pids[0]}"
    pids=("${pids[@]:1}")
  fi
done

# Wait for all background processes to finish
wait

# STEP 2: Resolve transitive dependencies recursively
echo "🔍 Resolving transitive dependencies..."
while true; do
  NEW_DEPS=()
  readarray -t deps < <(parse_pc_requires)
  
  if [[ ${#deps[@]} -eq 0 || ( ${#deps[@]} -eq 1 && -z "${deps[0]}" ) ]]; then
    break
  fi
  
  pids=()
  for dep in "${deps[@]}"; do
    if [[ -z "${VISITED[$dep]+x}" ]]; then
      echo "📦 Discovered transitive dep: $dep"
      VISITED[$dep]=1
      {
        fetch_pc_files "$dep"
      } &
      pids+=($!)
      NEW_DEPS+=("$dep")
      
      # Control number of parallel jobs
      if [[ ${#pids[@]} -ge $MAX_JOBS ]]; then
        wait "${pids[0]}"
        pids=("${pids[@]:1}")
      fi
    fi
  done
  
  # Wait for all background processes to finish
  wait
  
  [[ ${#NEW_DEPS[@]} -eq 0 ]] && break
  ALL_PACKAGES+=("${NEW_DEPS[@]}")
done

# STEP 3: Gather paths (optimized)
echo "📊 Gathering package paths..."
ALL_PATHS=()

# Do this in a single nix operation rather than one per package
all_pkgs_str=$(printf "nixpkgs#%s.dev nixpkgs#%s.out " "${ALL_PACKAGES[@]}" "${ALL_PACKAGES[@]}")
mapfile -t COMBINED_PATHS < <(nix path-info --inputs-from . --recursive $all_pkgs_str 2>/dev/null | sort -u)

if [[ ${#COMBINED_PATHS[@]} -gt 0 ]]; then
  ALL_PATHS=("${COMBINED_PATHS[@]}")
fi

echo "📂 Found ${#ALL_PATHS[@]} paths to process"

# STEP 4: Copy files (with parallelism)
echo "📄 Copying files from paths..."
process_path() {
  local STORE_PATH="$1"
  local temp_output="$TEMP_DIR/copy_log_$(basename "$STORE_PATH")"
  
  echo "📁 Processing $STORE_PATH" > "$temp_output"
  
  # Copy header files
  if [[ -d "$STORE_PATH/include" ]]; then
    cp -Lr --no-preserve=mode "$STORE_PATH/include/"* "$FINAL_STAGE/include/" 2>/dev/null || true
  fi
  
  # Copy libraries
  if [[ -d "$STORE_PATH/lib" ]]; then
    # Copy shared libraries
    find "$STORE_PATH/lib" -maxdepth 1 -type f -name '*.so*' \
      ! -name 'libc.so*' \
      ! -name 'libpthread.so*' \
      ! -name 'libm.so*' \
      ! -name 'libdl.so*' \
      ! -name 'libstdc++.so*' \
      -exec cp -L --no-preserve=mode {} "$FINAL_STAGE/lib/" \; 2>/dev/null || true
    
    # Copy symlinks as real symlinks
    find "$STORE_PATH/lib" -maxdepth 1 -type l -name '*.so*' \
      ! -name 'libc.so*' \
      ! -name 'libpthread.so*' \
      ! -name 'libm.so*' \
      ! -name 'libdl.so*' \
      ! -name 'libstdc++.so*' \
      -exec cp -P --no-preserve=mode {} "$FINAL_STAGE/lib/" \; 2>/dev/null || true
    
    # Copy static libraries
    find "$STORE_PATH/lib" -maxdepth 1 -type f -name '*.a' \
      ! -name 'libc.a' \
      -exec cp -L --no-preserve=mode {} "$FINAL_STAGE/lib/" \; 2>/dev/null || true
    
    # Copy pkg-config files
    if [[ -d "$STORE_PATH/lib/pkgconfig" ]]; then
      cp -Lr --no-preserve=mode "$STORE_PATH/lib/pkgconfig/"* "$FINAL_STAGE/lib/pkgconfig/" 2>/dev/null || true
    fi
    
    # KEY FIX: Copy C++ configuration headers from lib subdirectories
    # Find all library subdirectories with an include folder
    for lib_dir in $(find "$STORE_PATH/lib" -maxdepth 1 -mindepth 1 -type d); do
      lib_name=$(basename "$lib_dir")
      if [[ -d "$lib_dir/include" ]]; then
        # Create the target directory
        mkdir -p "$FINAL_STAGE/lib/$lib_name/include"
        # Copy all files from the include directory
        cp -Lr --no-preserve=mode "$lib_dir/include/"* "$FINAL_STAGE/lib/$lib_name/include/" 2>/dev/null || true
        echo "✅ Copied config headers from $lib_name" >> "$temp_output"
      fi
    done
  fi
  
  # Copy binaries and shared files
  if [[ -d "$STORE_PATH/bin" ]]; then
    cp -L --no-preserve=mode "$STORE_PATH/bin/"* "$FINAL_STAGE/bin/" 2>/dev/null || true
  fi
  
  if [[ -d "$STORE_PATH/share" ]]; then
    cp -LR --no-preserve=mode "$STORE_PATH/share/"* "$FINAL_STAGE/share/" 2>/dev/null || true
  fi
}

# Process paths in parallel
pids=()
for STORE_PATH in "${ALL_PATHS[@]}"; do
  process_path "$STORE_PATH" &
  pids+=($!)
  
  # Control number of parallel jobs
  if [[ ${#pids[@]} -ge $MAX_JOBS ]]; then
    wait "${pids[0]}"
    pids=("${pids[@]:1}")
  fi
done

# Wait for all background processes to finish
wait

# STEP 5: Patch pkg-config files (in parallel)
echo "🔧 Patching pkg-config files..."
patch_pc_file() {
  local pc_file="$1"
  [ -f "$pc_file" ] || return
  
  # If prefix is present, replace it; if not, add it to the top
  if grep -q '^prefix=' "$pc_file"; then
    sed -i 's|^prefix=.*|prefix=/app|' "$pc_file"
  else
    sed -i '1i prefix=/app' "$pc_file"
  fi

  # Set libdir
  if grep -q '^libdir=' "$pc_file"; then
    sed -i 's|^libdir=.*|libdir=/app/lib|' "$pc_file"
  else
    sed -i '2i libdir=/app/lib' "$pc_file"
  fi

  # Set includedir
  if grep -q '^includedir=' "$pc_file"; then
    sed -i 's|^includedir=.*|includedir=/app/include|' "$pc_file"
  else
    sed -i '3i includedir=/app/include' "$pc_file"
  fi

  # Remove problematic dependencies
  sed -i '/sysprof-capture-4/d' "$pc_file"
}

# Patch lib and share pkgconfig files in parallel
find "$FINAL_STAGE/lib/pkgconfig" "$FINAL_STAGE/share/pkgconfig" -name "*.pc" 2>/dev/null | \
while read -r pc_file; do
  patch_pc_file "$pc_file" &
  
  # Control number of parallel jobs
  current_jobs=$(jobs -p | wc -l)
  if [[ $current_jobs -ge $MAX_JOBS ]]; then
    wait -n
  fi
done

# Wait for all background patch jobs to finish
wait

# Print summary
echo "📋 Library directories found:"
find "$FINAL_STAGE/lib" -type d -name "include" | sort

echo "📋 Configuration headers found:"
find "$FINAL_STAGE/lib" -path "*/include/*config.h" | sort

# Clean up empty directories
find "$FINAL_STAGE" -type d -empty -delete

# Create the tarball
mkdir -p flatpak
tar -czf flatpak/nix-environment-dependencies.tar.gz -C "$FINAL_STAGE" .

# Clean up temporary directory
rm -rf "$TEMP_DIR"
echo "🎉 Done: flatpak/nix-environment-dependencies.tar.gz ready"
