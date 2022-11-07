import base.pkg.kusion_models.metadata.project
import base.pkg.kusion_models.metadata.stack

__META_APP_NAME = project.__META_PROJECT_NAME
__META_ENV_TYPE_NAME = stack.__META_STACK_NAME
__META_CLUSTER_NAME = option("cluster") or Undefined
