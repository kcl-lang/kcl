# Copyright 2021 The KCL Authors. All rights reserved.

import sys

import kclvm.api.version as version

USAGE = """\
usage: kclvm -m kclvm.api.version
       kclvm -m kclvm.api.version -checksum
       kclvm -m kclvm.api.version -h
"""

if __name__ == "__main__":
    if len(sys.argv) == 2 and (sys.argv[1] == "-h" or sys.argv[1] == "-help"):
        print(USAGE)
        sys.exit(0)

    if len(sys.argv) == 2 and sys.argv[1] == "-checksum":
        print(version.CHECKSUM)
        sys.exit(0)

    if len(sys.argv) > 1:
        print(USAGE)
        sys.exit(1)

    print(version.VERSION)
    sys.exit(0)
