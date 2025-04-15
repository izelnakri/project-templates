#!/bin/bash
# prepare-nix-deps.sh - Script for preparing multiple Nix dependencies into a single archive for flatpak-builder

# List of packages to prepare - add your dependencies here
PACKAGES=("jansson")

# Create a single temporary directory for all packages
TEMP_DIR=$(mktemp -d)
mkdir -p $TEMP_DIR/include
mkdir -p $TEMP_DIR/lib/pkgconfig
mkdir -p $TEMP_DIR/bin
mkdir -p $TEMP_DIR/share

echo "Preparing dependencies for flatpak..."

# Process each package
for PKG in "${PACKAGES[@]}"; do
  echo "Processing $PKG..."
  
  # Get paths from Nix
  DEV_PATH=$(nix eval --raw "nixpkgs#$PKG.dev" 2>/dev/null || echo "")
  LIB_PATH=$(nix eval --raw "nixpkgs#$PKG.out" 2>/dev/null || echo "")
  
  if [ -z "$DEV_PATH" ] && [ -z "$LIB_PATH" ]; then
    # Try without outputs if both failed
    SINGLE_PATH=$(nix eval --raw "nixpkgs#$PKG" 2>/dev/null || echo "")
    if [ -z "$SINGLE_PATH" ]; then
      echo "Warning: Could not find $PKG package, skipping"
      continue
    else
      echo "Using $PKG from: $SINGLE_PATH"
      DEV_PATH=$SINGLE_PATH
      LIB_PATH=$SINGLE_PATH
    fi
  else
    echo "Using $PKG dev from: $DEV_PATH"
    echo "Using $PKG lib from: $LIB_PATH"
  fi
  
  # Copy headers from dev package
  cp -L -r $DEV_PATH/include/* $TEMP_DIR/include/ 2>/dev/null || true
  
  # Copy libraries
  cp -L $LIB_PATH/lib/*.so* $TEMP_DIR/lib/ 2>/dev/null || true
  cp -L $DEV_PATH/lib/*.a $TEMP_DIR/lib/ 2>/dev/null || true
  cp -L $DEV_PATH/lib/*.la $TEMP_DIR/lib/ 2>/dev/null || true
  
  # Copy pkg-config files
  cp -L -r $DEV_PATH/lib/pkgconfig/* $TEMP_DIR/lib/pkgconfig/ 2>/dev/null || true
  
  # Copy binaries if they exist
  cp -L $LIB_PATH/bin/* $TEMP_DIR/bin/ 2>/dev/null || true
  
  # Copy share directory if it exists (carefully to avoid overwriting)
  if [ -d "$LIB_PATH/share" ]; then
    for dir in $LIB_PATH/share/*; do
      if [ -d "$dir" ]; then
        dir_name=$(basename "$dir")
        mkdir -p "$TEMP_DIR/share/$dir_name"
        cp -L -r "$dir"/* "$TEMP_DIR/share/$dir_name/" 2>/dev/null || true
      fi
    done
  fi
  
  # If no pkgconfig file exists but we have the library, create one
  if [ ! -f "$TEMP_DIR/lib/pkgconfig/$PKG.pc" ] && [ -f "$TEMP_DIR/lib/lib$PKG.so" ]; then
    VERSION=$(nix eval --raw "nixpkgs#$PKG.version" 2>/dev/null || echo "0.0.0")
    echo "Creating pkg-config file for $PKG with version $VERSION"
    
    cat > "$TEMP_DIR/lib/pkgconfig/$PKG.pc" << EOF
prefix=/app
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: $PKG
Description: Nix-provided $PKG library
Version: ${VERSION}
Libs: -L\${libdir} -l$PKG
Cflags: -I\${includedir}
EOF
  fi
done

# Fix pkg-config files to use /app prefix
if [ -d "$TEMP_DIR/lib/pkgconfig" ]; then
  for pc_file in $TEMP_DIR/lib/pkgconfig/*.pc; do
    if [ -f "$pc_file" ]; then
      sed -i 's|^prefix=.*|prefix=/app|g' "$pc_file"
    fi
  done
fi

# Make sure all files are writable (to avoid the permission issues with flatpak-builder)
chmod -R u+w $TEMP_DIR

# Remove empty directories
find $TEMP_DIR -type d -empty -delete

# Create a single archive with all dependencies
tar -czf "nix-deps.tar.gz" -C $TEMP_DIR .

# Clean up temporary directory
rm -rf $TEMP_DIR

echo "Created nix-deps.tar.gz with all dependencies"
