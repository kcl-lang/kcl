import base.pkg.kusion_models.kube.frontend.resource as res

import regex

schema Str2ResourceRequirements[resourcePara: str | res.Resource]:
    resourceStr?: str = resourcePara as str if typeof(resourcePara) == "str" else None
    resourceUnit?: res.Resource = resourcePara as res.Resource if typeof(resourcePara) == "Resource" else None
    schedulingResourceItems: [str] = resourceStr?.split(",") or []

    resource: [{str:}] = [{
        cpu = {
            requests = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[0])?.strip()
            limits = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[-1])?.strip()
        }
    } if "cpu" in item else ({
        memory = {
            requests = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[0])?.strip()
            limits = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[-1])?.strip()
        }
    } if "memory" in item else ({
        disk = {
            requests = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[0])?.strip()
            limits = (item.split("=")?[1] if len(item.split("=")) > 1 else item.split("<")?[-1])?.strip()
        }
    } if "disk" in item else Undefined)) for item in schedulingResourceItems]

    result: {str:} = {
        requests = {
            cpu = [r?.cpu?.requests for r in resource if r?.cpu?.requests]?[-1] or Undefined
            memory = [r?.memory?.requests for r in resource if r?.memory?.requests]?[-1] or Undefined
            "ephemeral-storage" = [r?.disk?.requests for r in resource if r?.disk?.requests]?[-1] or Undefined
        }
        limits = {
            cpu = [r?.cpu?.limits for r in resource if r?.cpu?.limits]?[-1] or Undefined
            memory = [r?.memory?.limits for r in resource if r?.memory?.limits]?[-1] or Undefined
            "ephemeral-storage" = [r?.disk?.limits for r in resource if r?.disk?.limits]?[-1] or Undefined
        }
    } if resourceStr else {
        requests = {
            cpu = str(resourceUnit.cpu)
            memory = str(resourceUnit.memory)
            "ephemeral-storage" = str(resourceUnit.disk) if resourceUnit.disk else Undefined
        }
        limits = {
            cpu = str(resourceUnit.cpu)
            memory = str(resourceUnit.memory)
            "ephemeral-storage" = str(resourceUnit.disk) if resourceUnit.disk else Undefined
        }
    }

    check:
        regex.match(result?.requests?.cpu, r"^([+-]?[0-9.]+)([m]*[-+]?[0-9]*)$") if result?.requests?.cpu, "cpu must match specific regular expression"
        regex.match(result?.requests?.memory, r"^([0-9]{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$") if result?.requests?.memory, "memory must match specific regular expression"
        regex.match(result?.requests?["ephemeral-storage"], r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$") if result?.requests?["ephemeral-storage"], "disk must match specific regular expression"
        regex.match(result?.limits?.cpu, r"^([+-]?[0-9.]+)([m]*[-+]?[0-9]*)$") if result?.limits?.cpu, "cpu must match specific regular expression"
        regex.match(result?.limits?.memory, r"^([0-9]{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$") if result?.limits?.memory, "memory must match specific regular expression"
        regex.match(result?.limits?["ephemeral-storage"], r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$") if result?.limits?["ephemeral-storage"], "disk must match specific regular expression"
