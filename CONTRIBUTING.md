# KCLVM Developing Guide

KCLVM follows a very standard Github development process, using Github tracker for issues and merging pull requests into master. If you would like to contribute something, or simply want to hack on the code this document should help you get started.

Before we accept a non-trivial patch or pull request we will need you to sign the Contributor License Agreement. Signing the contributor’s agreement does not grant anyone commits rights to the main repository, but it does mean that we can accept your contributions, and you will get an author credit if we do. Active contributors might be asked to join the core team and given the ability to merge pull requests.

## Install Dependencies

### macOS and OS X

+ `Python3.7+`
+ `Go 1.16+`
+ `Rust 2021 edition`
+ `openssl@1.1`

```
brew install openssl@1.1
```

### Linux

- `Go 1.16+`
- `Rust 2021 edition`
- `Python3 Building Dependencies`

For UNIX based systems, you can run:

```
yum groupinstall -y "Development Tools"
yum install -y gcc patch libffi-devel python-devel zlib-devel bzip2-devel ncurses-devel sqlite-devel 
yum install -y libpcap-devel xz-devel readline-devel tk-devel gdbm-devel db4-deve
yum -y install yum-utils
yum-builddep -y python3
yum install -y zlib* 
yum install -y openssl-devel
yum install -y glibc-static
```

On Debian, Ubuntu, and other apt based systems, you can run:

```
apt-get update

apt-get install -y git wget curl
apt-get install -y make gcc patch 
apt-get install -y python-dev libffi-dev
apt-get install -y zlib1g-dev ncurses-dev
```

### Docker

Use the image `kusionstack/kclvm-builder`, run:

```
make sh-in-docker
```

## Building and Testing

### Scripting

We provide a simple `run.sh` script to build and package with.

To build everything, run:

```
./run.sh -a build
```

Building includes two steps, which are `build-cpython` and `build-kclvm`. Alternatively, these steps can be invoked separately:

```
./run.sh -a build-cpython
./run.sh -a build-kclvm
```

Building KCL requires local ssl module. Use -s $yourLocalSSL to specify custom ssl path:

```
./run.sh -a build -s $your-local-ssl
./run.sh -a build-cpython -s $your-local-ssl
```

If -s option unset, default ssl path will be used:
Darwin: `$(brew --prefix openssl@1.1)`
Linux: /usr/lib64

To use KCL, add the path `_build/dist/{os}/kclvm/bin` to the `PATH` environment variable. Here, `{os}` stands for the operating system, which can be `Darwin`, `centos`, `ubuntu` or other types.

Then, you can run:

```
kcl hello.k
```

To perform testing, run:

```
./run.sh -a test
```

To update kclvm python libraries (./scripts/requirements.txt), run:

```
./run.sh -a build-kclvm
```

To update kclvm python code, run:

```
./run.sh -a update-kclvm
```

To build a tar file, run:

```
./run.sh -a release
```

Next, we can refer to [KCLVM README](./kclvm/README.md) for the next step of environment configuration.

## Using KCL

The specific user manual is as follows: [KCLVM User Manual](docs/cmd/README_KCLVM_USE.md)

## Code Structure

KCL has added the following files and directories:

+ `kclvm` -  The KCL compiler code.
+ `test` - All KCL test cases include regression tests and unit tests.
+ `scripts` -  The directory where additional scripts to build and test KCL resist.
+ `run.sh` - The script to perform operations such as building and testing.
+ `internal/kclvm_py` - KCLVM Python implementation, will be abandoned soon, please do not submit any code to it, it will be reorganized in the way of KCLVM Python SDK in the future.

During building and testing, the following directories can be generated:

+ `_build` - The directory to save building results including distributions.
