use crate::gpyrpc::ExecProgramArgs;

/// Transform the str with zero value into [`Option<String>`]
#[inline]
pub(crate) fn transform_str_para(para: &str) -> Option<String> {
    if para.is_empty() {
        None
    } else {
        Some(para.to_string())
    }
}

#[inline]
pub(crate) fn transform_exec_para(
    exec_args: &Option<ExecProgramArgs>,
) -> anyhow::Result<kclvm_runner::ExecProgramArgs> {
    Ok(match exec_args {
        Some(exec_args) => {
            let args_json = serde_json::to_string(exec_args)?;
            kclvm_runner::ExecProgramArgs::from_str(args_json.as_str())
        }
        None => kclvm_runner::ExecProgramArgs::default(),
    })
}
