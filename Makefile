# Copyright 2021 The KCL Authors. All rights reserved.

PROJECT_NAME = KCLVM

PWD:=$(shell pwd)

VERSION := $(shell cat VERSION)
BUILD_IMAGE:=kcllang/kcl-builder

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ${PWD}:/root/kclvm
RUN_IN_DOCKER+=-w /root/kclvm ${BUILD_IMAGE}

# ----------------
# KCLVM build
# ----------------

.PHONY: build
build:
	${PWD}/run.sh -a build

.PHONY: release
release:
	${PWD}/run.sh -a release

.PHONY: check
check:
	make -C ./kclvm check

.PHONY: test
test:
	make -C ./kclvm test

.PHONY: test-grammar
test-grammar:
	make -C ./kclvm test-grammar

.PHONY: fmt
fmt:
	make -C ./kclvm fmt

.PHONY: tag
tag:
	scripts/tag.sh v$(VERSION)

# ----------------
# Docker
# ----------------

sh-in-docker:
	${RUN_IN_DOCKER} bash
