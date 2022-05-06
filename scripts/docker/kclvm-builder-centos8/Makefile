# Copyright 2021 The KCL Authors. All rights reserved.

PWD:=$(shell pwd)
TIMESTAMP:=$(shell go run timestamp.go)

BUILDER_IMAGE:=reg.docker.inc.com/kusionstack/kclvm-builder-centos8

# export DOCKER_DEFAULT_PLATFORM=linux/amd64
# or
# --platform linux/amd64

RUN_IN_DOCKER:=docker run -it --rm --platform linux/amd64
RUN_IN_DOCKER+=-v ~/.ssh:/root/.ssh
RUN_IN_DOCKER+=-v ~/.gitconfig:/root/.gitconfig
RUN_IN_DOCKER+=-v ~/go/pkg/mod:/go/pkg/mod

kclvm-builder: crates.io-index
	docker build --platform linux/amd64 -t ${BUILDER_IMAGE} .
	@echo "ok"

crates.io-index:
	git clone  --mirror git@github.com:rust-lang/crates.io-index.git crates.io-index

vendor-kclvm:
	-rm -rf ${PWD}/../../kclvm/vendor

	${RUN_IN_DOCKER} -v ${PWD}/../..:/root/kclvm -w /root/kclvm/kclvm rust:1.54 cargo vendor --versioned-dirs
	mv ${PWD}/../../kclvm/vendor ./vendor-kclvm


vendor-kclvm-runtime:
	-rm -rf ${PWD}/../../kclvm/runtime/vendor

	${RUN_IN_DOCKER} -v ${PWD}/../..:/root/kclvm -w /root/kclvm/kclvm/runtime rust:1.54 cargo vendor --versioned-dirs
	mv ${PWD}/../../kclvm/runtime/vendor ./vendor-kclvm-runtime

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

sh-in-builder:
	${RUN_IN_DOCKER} -v ${PWD}/../..:/root/kclvm -w /root ${BUILDER_IMAGE} bash

clean:
