# Copyright 2021 The KCL Authors. All rights reserved.

from __future__ import annotations  # for python 3.7

from typing import List
from dataclasses import dataclass, field

# protobuf:
#
# message Tree {
#     string type = 1;
#     string token_value = 2;
#     repeated Tree children = 3;
#
#     int32 line = 101;
#     int32 column = 102;
#     int32 end_line = 103;
#     int32 end_column = 104;
# }


@dataclass
class Tree:
    type: str = ""
    token_value: str = ""
    children: List[Tree] = field(default_factory=lambda: [])

    line: int = 0
    column: int = 0
    end_line: int = 0
    end_column: int = 0
