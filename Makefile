# Copyright 2021 The KCL Authors. All rights reserved.

PROJECT_NAME = KCLVM

PWD:=$(shell pwd)

KCLVM_VERSION := $(shell cat VERSION)
BUILD_IMAGE:=kusionstack/kclvm-builder:centos7

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ~/go/pkg/mod:/go/pkg/mod
RUN_IN_DOCKER+=-v ${PWD}:/root/kclvm
RUN_IN_DOCKER+=-w /root/kclvm ${BUILD_IMAGE}

# ----------------
# KCLVM build
# ----------------

build:
	${PWD}/run.sh -a build

# ----------------
# Docker
# ----------------

sh-in-docker:
	${RUN_IN_DOCKER} bash
