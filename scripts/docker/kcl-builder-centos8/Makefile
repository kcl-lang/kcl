# Copyright 2021 The KCL Authors. All rights reserved.

PWD:=$(shell pwd)

BUILDER_IMAGE:=kcllang/kcl-builder:centos8

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm --platform linux/amd64
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ~/go/pkg/mod:/go/pkg/mod

kcl-builder:
	docker build --platform linux/amd64 -t ${BUILDER_IMAGE} .
	@echo "ok"

publish-builder:
	# https://docker.inc.com/
	# docker login --username=

	# make kcl-builder
	docker push ${BUILDER_IMAGE}
	@echo "push ${BUILDER_IMAGE} ok"

sh-in-builder:
	${RUN_IN_DOCKER} -v ${PWD}/../../..:/root/kclvm -w /root ${BUILDER_IMAGE} bash

clean:
