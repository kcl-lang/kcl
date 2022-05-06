## Env

+ OS: MacOS Manterey 12.1
+ CPU: Apple M1 pro, 10 cores
+ Mem: 16GB
+ Rust: cargo 1.59.0 (49d8809dc 2022-02-10)

## Prerequirements

1. Make sure you have arm64e-version homebrew installed @`/opt/homebrew`, otherwise install it ➡ 
    ```
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    export PATH=/opt/homebrew/bin:$PATH
    ```
2. Install openssl@1.1 ➡ brew install openssl@1.1
3. Check out openssl installed @`/opt/homebrew/opt/openssl@1.1/lib/libssl.1.1.dylib`

## Build LLVM from Source

1. Go to the folder where you want to build the LLVM source
2. Download LLVM 7.0.0 source from [http://releases.llvm.org/download.html](http://releases.llvm.org/download.html) and unpack source .tar.xz with tar -xJf [archive]
3. Create a directory where you want to build LLVM files ➡ mkdir build-llvm-7.0.0
4. Move into the build folder ➡ cd build-llvm-7.0.0; that's where you create `build.sh`.

```
#!usr/bin/env sh

LLVMSRC="<YOUR UNPACKED SOURCE PATH>"
. "$HOME/.cargo/env"
LLVMTARGET="<YOUR BUILD TARGET PATH>"


cmake \
    -G "Ninja" \
    -DCMAKE_INSTALL_PREFIX="$LLVMTARGET" \
    -DCMAKE_OSX_ARCHITECTURES='arm64' \
    -DCMAKE_C_COMPILER=`which clang` \
    -DCMAKE_CXX_COMPILER=`which clang++` \
    -DCMAKE_BUILD_TYPE="Release" \
    -DLLVM_TARGETS_TO_BUILD="AArch64" \
    -DLLVM_HOST_TRIPLE='aarch64-apple-darwin' \
    -DLLVM_DEFAULT_TARGET_TRIPLE='aarch64-apple-darwin' \
    -DLLVM_ENABLE_WERROR=FALSE \
    "$LLVMSRC"

cmake --build .
cmake --build . --target install
```

5. You may not have ninja installed, so if you have brew installed, you need to install it ninja with ➡ brew install `ninja`.
6. Run the script ➡ sh build.sh
7. Took about 10-15mins. Check out @`<YOUR BUILD TARGET PATH>`.

## Build KCLVM

1. Build KCLVM according to `kclvm/README.md`.
2. Took about 5mins.
3. Done!

## Notes

1. If you've brew-updating and github brew-submodule-download issue, you'd better use a mirror to speed up.
    ```
    cd "$(brew --repo)"
    git remote set-url origin https://mirrors.aliyun.com/homebrew/brew.git

    cd "$(brew --repo)/Library/Taps/homebrew/homebrew-core"
    git remote set-url origin https://mirrors.aliyun.com/homebrew/homebrew-core.git

    cd "$(brew --repo)/Library/Taps/homebrew/homebrew-cask"
git remote set-url origin https://mirrors.aliyun.com/homebrew/homebrew-cask.git

    echo 'export HOMEBREW_BOTTLE_DOMAIN=https://mirrors.aliyun.com/homebrew/homebrew-bottles' >> ~/.zshrc
source ~/.zshrc
    ```