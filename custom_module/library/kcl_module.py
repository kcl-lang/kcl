#!/usr/bin/python
from ansible.module_utils.basic import AnsibleModule
import kclvm.program.exec as kclvm_exec
import kclvm.vm.planner as planner

def run_kcl_command(module):
    result = {"changed": False}

    # Get parameters from Ansible playbook
    file_path = module.params.get("file_path")

    # Execute KCL command
    command_result = planner.plan(kclvm_exec.Run([file_path]).filter_by_path_selector())

    result["output"] = command_result

    return result

def main():
    module = AnsibleModule(
        argument_spec=dict(
            file_path=dict(type='str', required=True)
        ),
        supports_check_mode=True
    )

    result = run_kcl_command(module)
    module.exit_json(**result)

if __name__ == '__main__':
    main()
