# Copyright 2021 The KCL Authors. All rights reserved.

PROJECT_NAME = KCLVM

PWD:=$(shell pwd)

KCLVM_VERSION := $(shell cat VERSION)
BUILD_IMAGE:=kusionstack/kclvm-builder

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ~/go/pkg/mod:/go/pkg/mod
RUN_IN_DOCKER+=-v ${PWD}:/root/kclvm
RUN_IN_DOCKER+=-w /root/kclvm ${BUILD_IMAGE}

# --------------------
# KCLVM build and test
# --------------------

build:
	${PWD}/run.sh -a build

test:
	${PWD}/run.sh -a test

format:
	${PWD}/run.sh -a format

lint:
	${PWD}/run.sh -a lint-check

release:
	${PWD}/run.sh -a release

update-and-release:
	${PWD}/run.sh -a build-kclvm
	${PWD}/run.sh -a update-kclvm
	${PWD}/run.sh -a release

build-in-dcoker:
	${RUN_IN_DOCKER} make build

test-in-docker:
	${RUN_IN_DOCKER} make test

# ----------------
# Docker
# ----------------

sh-in-docker:
	${RUN_IN_DOCKER} bash
