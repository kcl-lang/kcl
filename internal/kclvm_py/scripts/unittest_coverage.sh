#!/bin/sh

# ---------------------------------------------------------------------------------
# Show the unit test coverage report
# For more info, see the
# [Python Coverage Documents](https://coverage.readthedocs.io/en/latest/)
# TODO: Using more mature tools and practices. e.g.
# https://github.com/CleanCut/green
# https://github.com/oldani/HtmlTestRunner
# ---------------------------------------------------------------------------------

src="$(realpath $(dirname $0))/../"
unittest_path=$src/test/test_units/
package_name=kclvm
xunit_file=TEST-kclvm.xml

# Install the dependency
kclvm -m pip install nose==1.3.7 
# Run test with coverage output
kclvm -m nose --cover-package $package_name --with-coverage --cover-xml --cover-html --cover-erase --with-xunit --xunit-file=$xunit_file --tests=$unittest_path
