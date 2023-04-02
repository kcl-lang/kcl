#!/bin/bash

# Display all commands before executing them.
set -o errexit
set -o errtrace

LLVM_VERSION=$1
LLVM_REPO_URL=${2:-https://github.com/llvm/llvm-project.git}
LLVM_CROSS="$3"

if [[ -z "$LLVM_REPO_URL" || -z "$LLVM_VERSION" ]]
then
  echo "Usage: $0 <llvm-version> <llvm-repository-url> [aarch64]"
  echo
  echo "# Arguments"
  echo "  llvm-version         The name of a LLVM release branch without the 'release/' prefix"
  echo "  llvm-repository-url  The URL used to clone LLVM sources (default: https://github.com/llvm/llvm-project.git)"
  echo "  aarch64              To cross-compile an aarch64 version of LLVM"

  exit 1
fi

# Clone the LLVM project.
if [ ! -d llvm-project ]
then
  git clone "$LLVM_REPO_URL" llvm-project
fi


cd llvm-project
git fetch origin
git checkout "release/$LLVM_VERSION"
git reset --hard origin/"release/$LLVM_VERSION"

# Create a directory to build the project.
mkdir -p build
cd build

# Create a directory to receive the complete installation.
mkdir -p install

# Adjust compilation based on the OS.
CMAKE_ARGUMENTS=""

case "${OSTYPE}" in
    darwin*) ;;
    linux*) ;;
    *) ;;
esac

# Adjust cross compilation
CROSS_COMPILE=""

case "${LLVM_CROSS}" in
    aarch64*) CROSS_COMPILE="-DLLVM_HOST_TRIPLE=aarch64-linux-gnu" ;;
    *) ;;
esac

# Run `cmake` to configure the project.
cmake \
  -G "Unix Makefiles" \
  -DCMAKE_BUILD_TYPE=MinSizeRel \
  -DCMAKE_INSTALL_PREFIX="/" \
  -DLLVM_ENABLE_PROJECTS="clang;lld" \
  -DLLVM_ENABLE_TERMINFO=OFF \
  -DLLVM_ENABLE_ZLIB=OFF \
  -DLLVM_INCLUDE_DOCS=OFF \
  -DLLVM_INCLUDE_EXAMPLES=OFF \
  -DLLVM_INCLUDE_GO_TESTS=OFF \
  -DLLVM_INCLUDE_TESTS=OFF \
  -DLLVM_INCLUDE_TOOLS=ON \
  -DLLVM_INCLUDE_UTILS=OFF \
  -DLLVM_OPTIMIZED_TABLEGEN=ON \
  -DLLVM_TARGETS_TO_BUILD="X86;AArch64" \
  "${CROSS_COMPILE}" \
  "${CMAKE_ARGUMENTS}" \
  ../llvm

# Showtime!
cmake --build . --config MinSizeRel
DESTDIR=destdir cmake --install . --strip --config MinSizeRel

# move usr/bin/* to bin/ or llvm-config will be broken
mv destdir/usr/bin/* destdir/bin/
