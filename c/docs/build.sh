#!/usr/bin/env bash
set -e

# Ensure the build directory is set up
if [ ! -d "build" ]; then
  echo "Setting up build directory..."
  meson setup build
fi

# Create docs directory if it doesn't exist
mkdir -p build/docs

# Extract include paths from compile commands or fall back to system include paths
echo "Extracting include paths..."
if [ -f "build/compile_commands.json" ]; then
  grep -o -- "-I[^ ]*" build/compile_commands.json | sort | uniq | sed 's/-I//g' > build/docs/all_include_paths.txt

  # Filter out invalid or unnecessary paths
  > build/docs/valid_include_paths.txt
  while read -r path; do
    if [ -d "$path" ] && [[ ! "$path" =~ \.p$ ]] && [[ ! "$path" =~ \.a ]] && [[ ! "$path" =~ \.so ]]; then
      echo "$path" >> build/docs/valid_include_paths.txt
    fi
  done < build/docs/all_include_paths.txt
else
  echo "Warning: compile_commands.json not found. Using system include paths only."
  echo "src" > build/docs/valid_include_paths.txt
fi

# Prepare paths to be added to Doxyfile
echo "Updating Doxyfile..."

# Gather source and documentation paths
echo "src" > build/docs/input_paths.txt
echo "README.md" >> build/docs/input_paths.txt
cat build/docs/valid_include_paths.txt >> build/docs/input_paths.txt

# Remove duplicates and empty lines, then update INPUT in Doxyfile
input_paths=$(sort -u build/docs/input_paths.txt | tr '\n' ' ')
sed -i '/^INPUT /d' docs/Doxyfile
echo "INPUT = $input_paths" >> docs/Doxyfile

# Update INCLUDE_PATH in Doxyfile
include_paths=$(sort -u build/docs/valid_include_paths.txt | tr '\n' ' ')
sed -i '/^INCLUDE_PATH /d' docs/Doxyfile
echo "INCLUDE_PATH = $include_paths" >> docs/Doxyfile

# Ensure the README.md is used as the main page in Doxyfile
sed -i '/^USE_MDFILE_AS_MAINPAGE /d' docs/Doxyfile
echo "USE_MDFILE_AS_MAINPAGE = README.md" >> docs/Doxyfile

# Build the documentation
echo "Building documentation..."
mkdir -p build/html
doxygen docs/Doxyfile

echo "Documentation generated in docs/html/"
