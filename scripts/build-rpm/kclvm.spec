# Build kclvm rpm after building and releasing
#     yum install -y rpm-build rpmdevtools
#     rpmdev-setuptree
#     cp ./_build/kclvm-centos-latest.tar.gz /root/rpmbuild/SOURCES/
#     rpmbuild -bb ./scripts/build-rpm/kclvm.spec
#     # upload /root/rpmbuild/RPMS/x86_64/kclvm-${version}-1.el7.x86_64.rpm
Name:           kclvm
Version:        0.4.6
Release:        1%{?dist}
Summary:        The KCL programming language and tools

License:        Apache-2.0
URL:            https://kcl-lang.io
Source0:        kclvm-centos-latest.tar.gz

%description

# Turn off the brp-python-bytecompile script
%global __os_install_post %(echo '%{__os_install_post}' | sed -e 's!/usr/lib[^[:space:]]*/brp-python-bytecompile[[:space:]].*$!!g')
%prep
%setup -q -c

%build
%define debug_package %{nil}

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/opt/kclvm
mkdir -p /opt/kclvm

cp -Rp ./kclvm/* %{buildroot}/opt/kclvm
cp -Rp ./kclvm/* /opt/kclvm

%clean
rm -rf %{buildroot}
rm -rf $RPM_BUILD_ROOT

%files
%defattr(-,root,root,-)
%dir /opt/kclvm
/opt/kclvm/*
