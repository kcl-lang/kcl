import kcl_plugin.project_context as ctx

# Get project information from project.yaml by kcl plugin
_projectContext = ctx.get_project_context()

__META_PROJECT_NAME = _projectContext['name']
