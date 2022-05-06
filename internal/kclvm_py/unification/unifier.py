# Copyright 2021 The KCL Authors. All rights reserved.

from typing import List
from dataclasses import dataclass
from collections import defaultdict

import kclvm.kcl.error as kcl_error

from .vertex import Vertex


NAME_NONE_BUCKET_KEY = "$name_none"


@dataclass
class UnifierConfig:
    """The vertex unification config"""

    check_unique: bool = False
    override: bool = False


class Unifier:
    def __init__(self, config: UnifierConfig = UnifierConfig()):
        self.config: UnifierConfig = config

    def unify(self, vertex: Vertex) -> Vertex:
        """The vertex unification function"""
        if not vertex or not isinstance(vertex, Vertex):
            return vertex
        # Using bucket map to check unique/merge option and store values
        bucket = defaultdict(list)
        for v in vertex.adjs or []:
            self.append_vertex_into_bucket(bucket, v)
        # Using the following configuration meta (filename/line/column) with the same name
        # to override the previous configuration e.g., `stack config -> base config`
        for k, v_list in bucket.items():
            if v_list:
                for j in range(len(v_list)):
                    v_list[j].meta = v_list[-1].meta
                # Merge vertices in the vertex list the with the same name
                bucket[k] = self.merge_vertices(v_list)
        # Merge the vertex adjs
        vertex.adjs = sum(bucket.values(), [])
        return vertex

    def merge_vertices(self, vertices: List[Vertex]) -> List[Vertex]:
        """Merge a vertex list with same names"""
        if not vertices or not isinstance(vertices, list):
            return []
        vertex_list = []
        # Merge all adjs in vertex with the same name,
        # if the adjs is None, append it into the vertex list
        total_adjs = sum([v.adjs or [] for v in vertices], [])
        is_unified = False
        meta_names = []
        for v in vertices:
            # If there are vertices in the list without adjs, they may have the
            # conflict values and put them into the vertex list
            # and deal the value conflict in VM
            if v.adjs is None and v.node:
                vertex_list.append(v)
            elif not is_unified:
                v.adjs = total_adjs
                vertex_list.append(self.unify(v))
                is_unified = True
            if v.config_meta.name and v.config_meta.name not in meta_names:
                if v.option.is_override:
                    meta_names = [v.config_meta.name]
                else:
                    meta_names.append(v.config_meta.name)
                if len(meta_names) >= 2 and v.option.is_union:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.CompileError_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=v.meta.filename,
                                line_no=v.meta.line,
                                col_no=v.meta.column,
                            )
                        ],
                        arg_msg=f"conflict unification types between {meta_names[-1]} and {meta_names[0]}",
                    )
        # Return the merged the vertex list
        return vertex_list

    def append_vertex_into_bucket(self, bucket: dict, v: Vertex):
        """Append the vertex into the bucket map with unique and override option"""
        if not isinstance(bucket, dict) or not isinstance(v, Vertex):
            return
        # Check unique key error
        if self.config.check_unique and len(bucket[v.name]) > 1 and v.option.is_unique:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=v.meta.filename,
                        line_no=v.node.line,
                        col_no=v.node.column,
                        end_col_no=v.node.end_column,
                    )
                ],
                arg_msg=f"config of '{v.name}' must be unique",
            )
        # Put the different value into the bucket with the same `name`.
        # Please note that then the vertex key is a runtime variable
        # such as string interpolation, the name is None e.g., `{"${name}": "Alice"}`
        # else the name is the key name such as `name` in `{"name": "Alice"}`
        if v.name is not None:
            # Override the value in the bucket
            if self.config.override:
                bucket[v.name] = [v]
            # Append the multiple values into the bucket and calculate these values in VM
            else:
                bucket[v.name].append(v)
        # Store the name none node including unpack expression and collection if expression
        else:
            bucket[NAME_NONE_BUCKET_KEY].append(v)
