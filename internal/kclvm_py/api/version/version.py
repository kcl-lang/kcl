# Copyright 2021 The KCL Authors. All rights reserved.

import os
from pathlib import Path

VERSION = "0.4.4"
CHECKSUM = Path(f"{os.path.dirname(__file__)}/checksum.txt").read_text().strip()
