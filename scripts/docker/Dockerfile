# Copyright 2021 The KCL Authors. All rights reserved.

FROM centos:centos7

# macOS M1 --platform linux/amd64
# try fix "Problem with the SSL CA cert (path? access rights?)"
# https://issueexplorer.com/issue/docker/for-mac/5912
# https://access.redhat.com/articles/2050743
RUN touch /etc/sysconfig/64bit_strstr_via_64bit_strstr_sse2_unaligned

# ---------------------------------------------------------------------------------
# Please note: The following steps are to install the dependency packages 
# needed to compile CPython for centos7, see the 
# [Python official website](https://devguide.python.org/setup/#install-dependencies) 
# for details. When the version of CPython used becomes higher, 
# please pay attention to update the installation dependencies. 
# ---------------------------------------------------------------------------------

# Some language environments and plug-ins related to development and compilation, 
# such as git, CPython compilation, etc.
RUN yum groupinstall -y "Development Tools"
# Compiler and tool chain required to compile CPython such as gcc, make, sqlite3, ctype, struct, etc.
RUN yum install -y gcc patch libffi-devel python-devel zlib-devel bzip2-devel ncurses-devel sqlite-devel 
RUN yum install -y libpcap-devel xz-devel readline-devel tk-devel gdbm-devel db4-deve
# Install the system libraries required by python3 for UNIX based systems
RUN yum -y install yum-utils
RUN yum-builddep -y python3
# The python zlib module dependency package is required when compiling the python source code, 
# in order to use the modules that require zlib, such as setuptools, etc.
RUN yum install -y zlib* 
# The python ssl module dependency package is required when compiling the python source code, 
# in order to use the modules that require ssl, such as pip3, twine, etc.
RUN yum install -y openssl-devel

# install which
RUN yum install -y which

# install static libc
RUN yum install -y glibc-static 

# install wget
RUN yum install -y wget

# install git-2.x
RUN yum -y install https://packages.endpoint.com/rhel/7/os/x86_64/endpoint-repo-1.7-1.x86_64.rpm
RUN yum -y install git

# rust
# https://forge.rust-lang.org/infra/other-installation-methods.html
RUN mkdir -p /root/download && cd /root/download \
  && wget https://static.rust-lang.org/dist/rust-1.56.1-x86_64-unknown-linux-gnu.tar.gz \
  && tar -zxvf rust-1.56.1-x86_64-unknown-linux-gnu.tar.gz \
  && cd rust-1.56.1-x86_64-unknown-linux-gnu \
  && ./install.sh
RUN rm -rf /root/download
RUN cargo -V

# python-3.7
RUN mkdir -p /root/download && cd /root/download \
  && wget http://npm.taobao.org/mirrors/python/3.7.10/Python-3.7.10.tgz \
  && tar -xzf Python-3.7.10.tgz \
  && cd Python-3.7.10 \
  && LANG=C.UTF-8 ./configure \
      --prefix=/usr/local/python3.7 \
      --enable-optimizations \
      --with-ssl \
  && make install
RUN ln -sf /usr/local/python3.7/bin/python3.7 /usr/bin/python3
RUN ln -sf /usr/local/python3.7/bin/python3.7 /usr/bin/python3.7

# python-3.7
# pkg-config --variable pc_path pkg-config
# pkg-config python-3.7 --cflags
# pkg-config python-3.7 --libs
RUN echo $'# See: man pkg-config\n\
Name: Python-3.7\n\
Description: Python-3.7 library\n\
Requires:\n\
Version: 3.7\n\
Libs.private: -lcrypt -lpthread -ldl -lutil\n\
Libs: -L/usr/local/python3.7/lib -lpython3.7m\n\
Cflags: -I/usr/local/python3.7/include/python3.7m\n'\
> /usr/lib64/pkgconfig/python-3.7.pc

# golang 1.16+
RUN mkdir -p /root/download && cd /root/download \
  && wget https://dl.google.com/go/go1.16.3.linux-amd64.tar.gz \
  && tar -zxvf go1.16.3.linux-amd64.tar.gz \
  && mv ./go /usr/local/go1.16.3
RUN ln -s /usr/local/go1.16.3/bin/go /usr/bin/go
RUN rm -rf /root/download

ENV GOPATH=/go
ENV GOLANG_VERSION=1.16.3
#ENV GOPROXY=https://goproxy.cn

RUN go get golang.org/x/lint/golint
RUN go get golang.org/x/tools/cmd/goimports
RUN go get honnef.co/go/tools/cmd/...

RUN go get github.com/t-yuki/gocover-cobertura
RUN go get github.com/jstemmer/go-junit-report

RUN rm -rf /go/pkg/mod
RUN rm -rf /go/pkg/sumdb

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
RUN echo 'Asia/Shanghai' >/etc/timezone

WORKDIR /root

CMD ["bash"]
