# Copyright 2021 The KCL Authors. All rights reserved.

PWD:=$(shell pwd)
TIMESTAMP:=$(shell go run timestamp.go)

BUILDER_IMAGE:=reg.docker.inc.com/kusionstack/kclvm-builder-ubuntu

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm --platform linux/amd64
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ~/go/pkg/mod:/go/pkg/mod

kclvm-builder-ubuntu: crates.io-index
	docker build --platform linux/amd64 -t ${BUILDER_IMAGE} .
	@echo "ok"

crates.io-index:
	git clone  --mirror git@github.com:rust-lang/crates.io-index.git crates.io-index

publish-builder:
	# https://docker.inc.com/
	# docker login --username= reg.docker.inc.com

	# make kclvm-builder
	docker push ${BUILDER_IMAGE}
	@echo "push ${BUILDER_IMAGE} ok"

publish-builder-tagged:
	# https://docker.inc.com/
	# docker login --username= reg.docker.inc.com

	# make kclvm-builder
	docker tag  ${BUILDER_IMAGE} ${BUILDER_IMAGE}:${TIMESTAMP}
	docker push ${BUILDER_IMAGE}:${TIMESTAMP}
	@echo "push ${BUILDER_IMAGE}:${TIMESTAMP} ok"

sh:
	${RUN_IN_DOCKER} -v ${PWD}/../..:/root/kclvm -w /root ${BUILDER_IMAGE} bash

clean:
