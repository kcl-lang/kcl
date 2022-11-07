import base.pkg.kusion_kubernetes.api.core.v1 as kubev1

ContainerFrontend2Kube = lambda c -> {str:} {
    # Kube variable
    _kubeResource = Undefined
    _kubeEnvList = Undefined
    _kubeEnvFromList = Undefined
    _kubePortList = Undefined
    _kubeReadinessProbe = Undefined
    _kubeLivenessProbe = Undefined
    _kubeStartupProbe = Undefined
    _kubeLifecycle = Undefined

    # Convert frontend resource to kube resource
    if c?.resource:
        _kubeResource = Str2ResourceRequirements(c.resource).result

    # Convert frontend env to kube env
    if c?.env:
        _kubeEnvList = [
            {
                name: e.name
                value: e.value
                valueFrom: {
                    fieldRef: e.valueFrom.fieldRef
                    configMapKeyRef: {
                        name: e.valueFrom.configMapKeyRef.name
                        key: e.valueFrom.configMapKeyRef.key
                    } if e.valueFrom.configMapKeyRef else Undefined
                    secretKeyRef: {
                        name: e.valueFrom.secretKeyRef.name
                        key: e.valueFrom.secretKeyRef.key
                    } if e.valueFrom.secretKeyRef else Undefined
                    resourceFieldRef: {
                        containerName: e.valueFrom.resourceFieldRef.containerName
                        resource: e.valueFrom.resourceFieldRef.resource
                        if e.valueFrom.resourceFieldRef.divisor:
                            divisor: str(e.valueFrom.resourceFieldRef.divisor)
                    } if e.valueFrom.resourceFieldRef else Undefined
                } if e.valueFrom else Undefined
            } for e in c.env
        ]

    # Convert frontend envFrom to kube envFrom
    if c?.envFrom:
        _kubeEnvFromList = [
            {
                configMapRef: {
                    name: ef.configMapRef
                } if ef.configMapRef else Undefined
                secretRef: {
                    name: ef.secretRef
                } if ef.secretRef else Undefined
            } for ef in c.envFrom
        ]

    # Convert kube ports to kube ports
    if c?.ports:
        _kubePortList = [
            {
                name: port.name
                containerPort: port.containerPort
                protocol: port.protocol
            } for port in c.ports
        ]

    # Convert frontend probe to kube probe
    if c?.readinessProbe or c?.livenessProbe or c?.startupProbe:
        _iacProbes = [
            c?.readinessProbe
            c?.livenessProbe
            c?.startupProbe
        ]
        _kubeProbes = [kubev1.Probe {
            exec: kubev1.ExecAction {
                command: _iacProbe.handler.command
            } if typeof(_iacProbe.handler) == "Exec" else Undefined
            httpGet: kubev1.HTTPGetAction {
                path: _iacProbe.handler.path
                port: _iacProbe.handler.port
                scheme: _iacProbe.handler.scheme
            } if typeof(_iacProbe.handler) == "Http" else Undefined
            tcpSocket: kubev1.TCPSocketAction {
                port: _iacProbe.handler.tcpSocket
            } if typeof(_iacProbe.handler) == "Tcp" else Undefined
            failureThreshold: _iacProbe.failureThreshold
            initialDelaySeconds: _iacProbe.initialDelaySeconds
            periodSeconds: _iacProbe.periodSeconds
            successThreshold: _iacProbe.successThreshold
            timeoutSeconds: _iacProbe.timeoutSeconds
        } if _iacProbe else Undefined for _iacProbe in _iacProbes]
        _kubeReadinessProbe = _kubeProbes[0]
        _kubeLivenessProbe = _kubeProbes[1]
        _kubeStartupProbe = _kubeProbes[2]

    # Convert frontend lifecycle to kube lifecycle
    if c?.lifecycle:
        _kubeLifecycle = {
            preStop: {
                exec: kubev1.ExecAction {
                    command: c.lifecycle.preStop.command
                } if typeof(c.lifecycle.preStop) == "Exec" else Undefined
                httpGet: kubev1.HTTPGetAction {
                    path: c.lifecycle.preStop.path
                    port: c.lifecycle.preStop.port
                    scheme: c.lifecycle.preStop.scheme
                } if typeof(c.lifecycle.preStop) == "Http" else Undefined
                tcpSocket: kubev1.TCPSocketAction {
                    port: c.lifecycle.preStop.tcpSocket
                } if typeof(c.lifecycle.preStop) == "Tcp" else Undefined
            } if c.lifecycle.preStop else Undefined
            postStart: {
                exec: kubev1.ExecAction {
                    command: c.lifecycle.postStart.command
                } if typeof(c.lifecycle.preStop) == "Exec" else Undefined
                httpGet: kubev1.HTTPGetAction {
                    path: c.lifecycle.postStart.path
                    port: c.lifecycle.postStart.port
                    scheme: c.lifecycle.postStart.scheme
                } if typeof(c.lifecycle.preStop) == "Http" else Undefined
                tcpSocket: kubev1.TCPSocketAction {
                    port: c.lifecycle.postStart.tcpSocket
                } if typeof(c.lifecycle.preStop) == "Tcp" else Undefined
            } if c.lifecycle.postStart else Undefined
        }

    # Construct result
    {
        name: c?.name
        image: c?.image
        imagePullPolicy: c?.imagePullPolicy if "imagePullPolicy" in c else Undefined
        resources: _kubeResource
        env: _kubeEnvList
        envFrom: _kubeEnvFromList
        command: c?.command
        args: c?.args
        ports: _kubePortList
        securityContext: c?.securityContext
        readinessProbe: _kubeReadinessProbe
        livenessProbe: _kubeLivenessProbe
        startupProbe: _kubeStartupProbe
        lifecycle: _kubeLifecycle
        workingDir: c?.workingDir
        terminationMessagePath: c?.terminationMessagePath if "terminationMessagePath" in c else Undefined
        terminationMessagePolicy: c?.terminationMessagePolicy if "terminationMessagePolicy" in c else Undefined
    }
}
