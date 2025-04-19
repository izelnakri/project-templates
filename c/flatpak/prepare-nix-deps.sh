#!/usr/bin/env bash
set -euo pipefail

# INITIAL_PACKAGES=("jansson" "gtk4" "glib" "lerc") # Add packages as needed
INITIAL_PACKAGES=("jansson") # NOTE: this doesnt need gtk4, glib on gnome sdk & runtime
TEMP_DIR=$(mktemp -d)
FINAL_STAGE="$TEMP_DIR/final"
PC_TEMP="$TEMP_DIR/pcfiles"

mkdir -p "$FINAL_STAGE/include" "$FINAL_STAGE/lib/pkgconfig" "$FINAL_STAGE/bin" "$FINAL_STAGE/share" "$PC_TEMP"

declare -A VISITED
ALL_PACKAGES=("${INITIAL_PACKAGES[@]}")

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
    | sort -u
}

function get_all_needed_pc_files() {
  grep -h -E '^Requires(\.private)?\s*:' "$FINAL_STAGE/lib/pkgconfig/"*.pc 2>/dev/null \
    | cut -d: -f2- \
    | tr ',' '\n' \
    | awk '{print $1}' \
    | sed 's/[><=].*//' \
    | grep -v '^$' \
    | sort -u
}

function pc_file_exists() {
  [[ -f "$FINAL_STAGE/lib/pkgconfig/$1.pc" ]] || [[ -f "$PC_TEMP/$1.pc" ]]
}

function try_fetch_and_copy_pc_file() {
  local dep="$1"
  if pc_file_exists "$dep"; then return 0; fi
  fetch_pc_files "$dep"
  if ls "$PC_TEMP/$dep.pc" > /dev/null 2>&1; then
    echo "📦 Copied $dep.pc"
    return 0
  else
    echo "❌ Still missing $dep.pc"
    return 1
  fi
}

# STEP 1: Fetch initial PC files
for pkg in "${INITIAL_PACKAGES[@]}"; do
  fetch_pc_files "$pkg"
  VISITED[$pkg]=1
done

# STEP 2: Resolve transitive dependencies recursively
while true; do
  NEW_DEPS=()
  for dep in $(parse_pc_requires); do
    if [[ -z "${VISITED[$dep]+x}" ]]; then
      echo "📦 Discovered transitive dep: $dep"
      VISITED[$dep]=1
      fetch_pc_files "$dep"
      NEW_DEPS+=("$dep")
    fi
  done
  [[ ${#NEW_DEPS[@]} -eq 0 ]] && break
  ALL_PACKAGES+=("${NEW_DEPS[@]}")
done

# STEP 3: Copy closure
ALL_PATHS=()
for pkg in "${ALL_PACKAGES[@]}"; do
  echo "📦 Resolving closure for $pkg"
  dev_path=$(resolve_nix_path "$pkg.dev")
  out_path=$(resolve_nix_path "$pkg.out")

  [[ -n "$dev_path" ]] && mapfile -t dev_paths < <(nix path-info --recursive "$dev_path" 2>/dev/null || echo "") 
  if [[ -n "$dev_path" && ${#dev_paths[@]} -gt 0 && -n "${dev_paths[0]}" ]]; then
    ALL_PATHS+=("${dev_paths[@]}")
  fi
  
  [[ -n "$out_path" ]] && mapfile -t out_paths < <(nix path-info --recursive "$out_path" 2>/dev/null || echo "")
  if [[ -n "$out_path" && ${#out_paths[@]} -gt 0 && -n "${out_paths[0]}" ]]; then
    ALL_PATHS+=("${out_paths[@]}")
  fi
done

readarray -t ALL_PATHS < <(printf "%s\n" "${ALL_PATHS[@]}" | sort -u)

for STORE_PATH in "${ALL_PATHS[@]}"; do
  echo "📁 Copying from $STORE_PATH"
  
  # Copy header files
  if [[ -d "$STORE_PATH/include" ]]; then
    cp -Lr --no-preserve=mode "$STORE_PATH/include/"* "$FINAL_STAGE/include/" 2>/dev/null || true
  fi
  
  # Copy libraries
  if [[ -d "$STORE_PATH/lib" ]]; then
    # Copy all shared libraries
    find "$STORE_PATH/lib" -maxdepth 1 -type f -name '*.so*' \
      ! -name 'libc.so*' \
      ! -name 'libpthread.so*' \
      ! -name 'libm.so*' \
      ! -name 'libdl.so*' \
      ! -name 'libstdc++.so*' \
      -exec cp -L --no-preserve=mode {} "$FINAL_STAGE/lib/" \; 2>/dev/null || true
    
    # Copy symlinks as real symlinks, not resolved files
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
  fi
  
  # Copy binaries and shared files
  if [[ -d "$STORE_PATH/bin" ]]; then
    cp -L --no-preserve=mode "$STORE_PATH/bin/"* "$FINAL_STAGE/bin/" 2>/dev/null || true
  fi
  
  if [[ -d "$STORE_PATH/share" ]]; then
    cp -LR --no-preserve=mode "$STORE_PATH/share/"* "$FINAL_STAGE/share/" 2>/dev/null || true
  fi
done

# STEP 4: Fix library symlinks and ensure standard library naming convention -> NOTE: is this part really needed?
cd "$FINAL_STAGE/lib"
echo "🔧 Checking library symlinks..."

# Get list of all real libraries (not symlinks)
REAL_LIBS=$(find . -type f -name "*.so*" | sort)

# For each real library, ensure we have the proper symlinks
for lib in $REAL_LIBS; do
  # Get the SONAME for this library
  SONAME=$(objdump -p "$lib" 2>/dev/null | grep SONAME | awk '{print $2}' || echo "")
  
  # If no SONAME, try to generate one from the filename
  if [[ -z "$SONAME" ]]; then
    BASE_NAME=$(basename "$lib")
    # Extract the base name and version
    if [[ "$BASE_NAME" =~ (lib[^.]*)\.so\.([0-9]+)(\.[0-9]+)* ]]; then
      LIB_NAME="${BASH_REMATCH[1]}"
      MAJOR_VERSION="${BASH_REMATCH[2]}"
      SONAME="${LIB_NAME}.so.${MAJOR_VERSION}"
      echo "📚 Generated SONAME for $lib: $SONAME"
    fi
  else
    echo "📚 Found SONAME for $lib: $SONAME"
  fi
  
  # If we have a SONAME, ensure the symlink exists
  if [[ -n "$SONAME" && ! -e "$SONAME" && "$lib" != "./$SONAME" ]]; then
    ln -sf "$(basename "$lib")" "$SONAME"
    echo "🔗 Created symlink $SONAME -> $(basename "$lib")"
  fi
  
  # Also create the unversioned .so symlink for linking
  if [[ "$SONAME" =~ (lib[^.]*)\.so\..* ]]; then
    LIB_BASE="${BASH_REMATCH[1]}"
    UNVERSIONED="${LIB_BASE}.so"
    if [[ ! -e "$UNVERSIONED" ]]; then
      ln -sf "$SONAME" "$UNVERSIONED"
      echo "🔗 Created unversioned symlink $UNVERSIONED -> $SONAME"
    fi
  fi
done

cd - > /dev/null

# STEP 5: Patch all pkg-config files
for pc_file in "$FINAL_STAGE/lib/pkgconfig/"*.pc; do
  [ -f "$pc_file" ] || continue
  sed -i 's|^prefix=.*|prefix=/app|g' "$pc_file"
  # Remove problematic dependencies
  sed -i '/sysprof-capture-4/d' "$pc_file"
done

# STEP 6: Final check for missing deps. NOTE:Why is this part needed?
while true; do
  MISSING=()
  for dep in $(get_all_needed_pc_files); do
    if ! pc_file_exists "$dep"; then
      MISSING+=("$dep")
    fi
  done

  if [[ ${#MISSING[@]} -eq 0 ]]; then
    echo "✅ All dependencies satisfied!"
    break
  fi

  echo "🔁 Trying to fetch missing deps: ${MISSING[*]}"
  NEW_FOUND=0
  for missing_dep in "${MISSING[@]}"; do
    if try_fetch_and_copy_pc_file "$missing_dep"; then
      ALL_PACKAGES+=("$missing_dep")
      NEW_FOUND=1
    fi
  done

  if [[ $NEW_FOUND -eq 0 ]]; then
    echo "❌ Could not find some .pc files: ${MISSING[*]}"
    break
  fi
done

# Print library directory contents for verification
echo "📋 Contents of $FINAL_STAGE/lib:"
ls -la "$FINAL_STAGE/lib" | grep -v "total"

# Clean up empty directories
find "$FINAL_STAGE" -type d -empty -delete

# Create the tarball
tar -czf flatpak/nix-deps.tar.gz -C "$FINAL_STAGE" .

# Clean up temporary directory
rm -rf "$TEMP_DIR"
echo "🎉 Done: flatpak/nix-deps.tar.gz ready"
