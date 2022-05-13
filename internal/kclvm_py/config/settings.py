# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
from typing import List

import kclvm.config
import kclvm.internal.util as util
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2


KCL_MOD_PATH_ENV = "${KCL_MOD}"


def load_settings_files(
    work_dir: str, files: List[str]
) -> pb2.LoadSettingsFiles_Result:
    """Load KCL CLI config from the setting files.

    Parameter
    ---------
    work_dir : str
        The kcl run work directory.
    files:
        The setting YAML files.

    Returns
    -------
    result: LoadSettingsFiles_Result
        The merged kcl singleton config.
    """
    from kclvm.compiler.vfs import GetPkgRoot

    if not files:
        return pb2.LoadSettingsFiles_Result(
            kcl_cli_configs=pb2.CliConfig(), kcl_options=[]
        )
    key_value_pairs = [
        pb2.KeyValuePair(key=k, value=v)
        for k, v in util.merge_option_same_keys(
            kclvm.config.KCLCLISettingAction().deal(files)
        ).items()
    ]
    if work_dir or kclvm.config.current_path:
        files = [
            str(
                pathlib.Path(work_dir)
                .joinpath(
                    str(x).replace(
                        KCL_MOD_PATH_ENV,
                        GetPkgRoot(work_dir or kclvm.config.current_path or files[0])
                        or "",
                    )
                )
                .resolve()
            )
            for x in kclvm.config.input_file
        ]
    return pb2.LoadSettingsFiles_Result(
        kcl_cli_configs=pb2.CliConfig(
            files=files,
            output=kclvm.config.output,
            overrides=kclvm.config.overrides,
            path_selector=kclvm.config.path_selector,
            strict_range_check=kclvm.config.strict_range_check,
            disable_none=kclvm.config.disable_none,
            verbose=kclvm.config.verbose,
            debug=kclvm.config.debug,
        ),
        kcl_options=key_value_pairs,
    )
