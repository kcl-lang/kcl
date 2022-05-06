# Copyright 2021 The KCL Authors. All rights reserved.

default:
	swig -c++ -python kclvm_plugin.i
	python3 -m black .

	cd ../runtime && cargo fmt && cargo build --release

	kclvm setup.py build_ext --inplace
	kclvm setup.py install_lib

	@echo ------------------
	cd demo && kclvm hello.py

swig:
	swig -c++ -python kclvm_plugin.i

clean:
	-rm  -rf build *.so *.dll
