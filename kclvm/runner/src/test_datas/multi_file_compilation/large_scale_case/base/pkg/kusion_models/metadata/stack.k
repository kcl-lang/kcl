import kcl_plugin.project_context as ctx

# Get stack information from stack.yaml by kcl plugin
_stackContext = ctx.get_stack_context()

__META_STACK_NAME = _stackContext['name']
