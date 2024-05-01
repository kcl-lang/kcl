# Copyright 2021 The KCL Authors. All rights reserved.

FROM fedora:39

# Fix SSL CA cert issue
RUN touch /etc/sysconfig/64bit_strstr_via_64bit_strstr_sse2_unaligned

# Fix mirrorlist issue
RUN sed -i -e "s|metalink=|#metalink=|g" /etc/yum.repos.d/fedora*.repo
RUN sed -i -e "s|#baseurl=http://download.example/pub/fedora/linux|baseurl=https://dl.fedoraproject.org/pub/fedora/linux|g" /etc/yum.repos.d/fedora*.repo

# install necessary packages
RUN dnf -y install make which wget git ca-certificates clang llvm-devel libffi-devel python3-devel

# rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# go
RUN mkdir -p /root/download && cd /root/download \
    && wget https://dl.google.com/go/go1.20.5.linux-amd64.tar.gz \
    && tar -zxvf go1.20.5.linux-amd64.tar.gz \
    && mv ./go /usr/local/go1.20.5
RUN ln -sf /usr/local/go1.20.5/bin/go /usr/bin/go
RUN rm -rf /root/download
ENV GOPATH=/go
ENV GOLANG_VERSION=1.20.5

# go tools
RUN go install golang.org/x/lint/golint@latest \
    && go install golang.org/x/tools/cmd/goimports@latest \
    && go install github.com/t-yuki/gocover-cobertura@latest \
    && go install github.com/jstemmer/go-junit-report@latest

RUN python3 -m pip install pytest

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

RUN mkdir -p /root/.cargo && touch /root/.cargo/env

WORKDIR /root

CMD ["bash"]
