import sys
import kclvm.kcl.error as kcl_error
import os

cwd = os.path.dirname(os.path.realpath(__file__))

kcl_error.print_kcl_error_message(
    kcl_error.get_exception(
        err_type=kcl_error.ErrType.TypeError_Compile_TYPE,
        file_msgs=[
            kcl_error.ErrFileMsg(
                filename=cwd + "/main.k",
                line_no=4,
                col_no=5,
                arg_msg="got {str(podAntiAffinity):{str(preferredDuringSchedulingIgnoredDuringExecution):[{str(weight)|str(podAffinityTerm):int(100)|{str(labelSelector)|str(topologyKey):str(kubernetes.io/hostname)|{str(matchExpressions):[{str(key)|str(operator)|str(values):str(cluster.k8s/app-name)|str(In)|[str]}]}}}]}}"
            )
        ],
        arg_msg="expect {str:str}, got {str(podAntiAffinity):{str(preferredDuringSchedulingIgnoredDuringExecution):[{str(weight)|str(podAffinityTerm):int(100)|{str(labelSelector)|str(topologyKey):str(kubernetes.io/hostname)|{str(matchExpressions):[{str(key)|str(operator)|str(values):str(cluster.k8s/app-name)|str(In)|[str]}]}}}]}}"
    ),
    file=sys.stdout
)

