# Copyright 2021 The KCL Authors. All rights reserved.

FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install python3 python3-pip -y

# Install python3 and pip

COPY _build/dist/ubuntu/kclvm /kclvm/

RUN chmod +x /kclvm/bin/kcl
RUN chmod +x /kclvm/bin/kcl-doc
RUN chmod +x /kclvm/bin/kcl-fmt
RUN chmod +x /kclvm/bin/kcl-lint
RUN chmod +x /kclvm/bin/kcl-plugin
RUN chmod +x /kclvm/bin/kcl-test
RUN chmod +x /kclvm/bin/kcl-vet
RUN chmod +x /kclvm/bin/kclvm
RUN chmod +x /kclvm/bin/kclvm_cli
RUN /kclvm/bin/kcl

ENV PATH="/kclvm/bin:${PATH}"
ENV LANG=en_US.utf8

CMD ["bash"]
