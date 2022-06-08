# Quick Start

This documentation is *NOT* intended to be comprehensive; it is meant to be a quick guide for the most useful things. For more information, see the develop guide in its entirety.

## Asking Questions

Before asking a question, make sure you have:

- Searched open and closed:
  - [KCLVM GitHub Issues](https://github.com/KusionStack/KCLVM/issues?utf8=%E2%9C%93&q=is%3Aissue)

- Read the documentations:
  - [KCLVM Documents](https://kusionstack.io/docs/reference/lang/lang/tour)
  - [KCLVM Readme](https://github.com/KusionStack/KCLVM)

If you have any questions about `KCLVM`, you are welcome to ask your questions in [KCLVM Github Issues](https://github.com/KusionStack/KCLVM/issues). When you ask a question, please describe the details as clearly as possible so that others in the KCLVM community can understand, and you *MUST* be polite and avoid personal attack and avoid not objective comparison with other projects.

## Cloning and Building `KCLVM`

### System Requirements

The following hardware is recommended.

- 10GB+ of free disk space.
- 4GB+ RAM
- 2+ cores

### Dependencies

#### Docker

- `docker`

In the top level of the `KusionStack/KCLVM` repo and run:

```sh
make sh-in-docker
```

Using a docker image is our recommended way, of course, you can also configure your local development environment according to the following content.

#### macOS and OS X

- `git`
- `Python 3.7+`
- `Rust 1.60+`
- `LLVM 12`

You'll need LLVM installed and `llvm-config` in your `PATH`. Just download from [LLVM 12](https://releases.llvm.org/download.html) or install `llvm@12` using `brew`.

```sh
brew install llvm@12
```

Add the LLVM installation location to `LLVM_SYS_120_PREFIX` and the `$PATH`.

```
export LLVM_SYS_120_PREFIX=<your LLVM 12 install location>
export PATH=<your LLVM 12 install location>/bin:$PATH
```

#### Linux

- `git`
- `Rust 1.60+`
- `Python3 Building Dependencies`
- `LLVM 12`

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

# clang-12 & llvm-12
RUN yum -y install clang
RUN clang --version
RUN yum -y install llvm-devel
RUN yum -y install libffi-devel
RUN ln -s /usr/lib64/libtinfo.so.6 /usr/lib64/libtinfo.so
```

On Debian, Ubuntu, and other apt based systems, you can run:

```
apt-get update

apt-get install -y git wget curl
apt-get install -y make gcc patch 
apt-get install -y python-dev libffi-dev
apt-get install -y zlib1g-dev ncurses-dev build-essential libncurses5-dev libgdbm-dev libnss3-dev libssl-dev libreadline-dev libffi-dev

# clang-12 & llvm-12
RUN apt-get install -y clang-12 lld-12
RUN ln -sf /usr/bin/clang-12   /usr/bin/clang
RUN ln -sf /usr/bin/wasm-ld-12 /usr/bin/wasm-ld
```

#### Windows

- `git`
- `Rust 1.60+`
- `Python 3.7+`
- `LLVM 12`

Please add the LLVM installation location to `LLVM_SYS_120_PREFIX` and the `$PATH`.

### Cloning

You can just do a normal git clone:

```sh
git clone https://github.com/KusionStack/KCLVM.git
cd KCLVM
```

### Building

In the top level of the `KusionStack/KCLVM` repo and run:

```sh
./run.sh -a build
```

### Testing

In the top level of the `KusionStack/KCLVM` repo and run:

```
./run.sh -a test
```

See the chapters on building and testing for more details.

## Contributor Procedures

### Create an Issue

Every change should be accompanied by a dedicated tracking issue for that change. The main text of this issue should describe the change being made, with a focus on what users must do to fix their code. The issue should be approachable and practical; it may make sense to direct users to some other issue for the full details. The issue also serves as a place where users can comment with questions or other concerns.

When you open an issue on the `KusionStack/KCLVM` repo, you need to to choose an issue template on this [page](https://github.com/KusionStack/KCLVM/issues/new/choose), you can choose a template according to different situations and fill in the corresponding content, and you also need to select appropriate labels for your issue to help classify and identify.

### Create a PR

When you open a PR on the `KusionStack/KCLVM` repo, you need to assign reviewers in the [KCL Dev Team](https://github.com/orgs/KusionStack/teams/kcl-dev) list, and reviewers are the persons that will approve the PR to be tested and merged.

Please note that all code changes in the KCLVM project require corresponding comments and tests. For more code and test writing details, please see the chapters on code of conduct and testing.

Besides, all PRs need to have corresponding issues tracking, and need to add appropriate labels and milestone information.

#### Bug Fixes or "Normal" Code Changes

For most PRs, no special procedures are needed. You can just open an issue and a PR, and it will be reviewed, approved, and merged. This includes most bug fixes, refactorings, and other user-invisible changes.

Also, note that it is perfectly acceptable to open WIP PRs or GitHub Draft PRs. Some people prefer to do this so they can get feedback along the way or share their code with a collaborator. Others do this so they can utilize the CI to build and test their PR (e.g. if you are developing on a laptop).

#### New Features

In order to implement a new feature, usually you will need to go through [the KEP process](https://github.com/KusionStack/KEP) to propose a design, have discussions, etc.

After a feature is approved to be added, a tracking issue is created on the `KusionStack/KCLVM` repo, which tracks the progress towards the implementation of the feature, any bugs reported, and eventually stabilization. The feature then can be implemented.
