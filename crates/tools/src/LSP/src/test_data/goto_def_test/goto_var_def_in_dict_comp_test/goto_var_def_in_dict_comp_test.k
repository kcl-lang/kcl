capabilities = option("params").capabilities or ["SETUID", "SETFCAP"]
items1 = [item | {
    if item.kind == "Pod":
        spec.containers: [{
            "securityContext": {"capabilities": {"add" += [cc] if cc not in (container?.securityContext?.capabilities?.drop or []) else [] for cc in capabilities}}
        } for container in item.spec.containers]
    
} for item in option("items")]
