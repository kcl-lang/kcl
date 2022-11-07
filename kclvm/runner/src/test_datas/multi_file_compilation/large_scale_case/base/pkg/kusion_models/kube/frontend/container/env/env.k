import regex
import units

schema Env:
    """ Env represents an environment variable present in a Container.

    Attributes
    ----------
    name: str, default is Undefined, required.
        A Container-level attribute.
        The env name. This must be a C_IDENTIFIER.
    value: str, default is Undefined, optional.
        A Container-level attribute.
        The simple literal value.
    valueFrom: EnvValueFrom, default is Undefined, optional.
        A Container-level attribute.
        The ref source of this env.
    """

    name: str
    value?: str
    valueFrom?: EnvValueFrom

    check:
        len(name) <= 63, "a valid env name must be no more than 63 characters"
        regex.match(name, r"[A-Za-z_][A-Za-z0-9_]*"), "a valid env name must start with alphabetic character or '_', followed by a string of alphanumeric characters or '_'"
        not valueFrom if value, "valueFrom may not be specified when `value` is not empty"

schema EnvValueFrom:
    """ EnvValueFrom represents the source of the value of an Env.

    Attributes
    ----------
    fieldRef: ObjectFieldSelector, default is Undefined, optional.
        A Container-level attribute.
        Selects a key of a field.
    configMapKeyRef: ObjectKeySelector, default is Undefined, optional.
        A Container-level attribute.
        Selects a key of a ConfigMap.
    secretKeyRef: ObjectKeySelector, default is Undefined, optional.
        A Container-level attribute.
        Selects a key of a secret.
    resourceFieldRef: ResourceFieldSelector, default is Undefined, optional.
        A Container-level attribute.
        Selects a resource of the container: only resources limits and requests 
        (limits.cpu, limits.memory, limits.ephemeral-storage, requests.cpu, requests.memory and requests.ephemeral-storage) are currently supported.
    """

    fieldRef?: ObjectFieldSelector
    configMapKeyRef?: ObjectKeySelector
    secretKeyRef?: ObjectKeySelector
    resourceFieldRef?: ResourceFieldSelector

    check:
        (fieldRef and not configMapKeyRef and not secretKeyRef and not resourceFieldRef) or \
        (not fieldRef and configMapKeyRef and not secretKeyRef and not resourceFieldRef) or \
        (not fieldRef and not configMapKeyRef and secretKeyRef and not resourceFieldRef) or \
        (not fieldRef and not configMapKeyRef and not secretKeyRef and resourceFieldRef), "must specify one of: `fieldRef`, `configMapKeyRef`, `secretKeyRef` or `resourceFieldRef`"

schema ObjectKeySelector:
    """ ObjectKeySelector contains enough information to let you locate the referenced object.

    Attributes
    ----------
    name: str, default is Undefined, required.
        A Container-level attribute.
        The name of object, typically a ConfigMap or Secret name.
    key: str, default is Undefined, required.
        A Container-level attribute.
        The key of the object to select from.
    """

    name: str
    key: str

schema ObjectFieldSelector:
    """ ObjectFieldSelector contains enough information to let you select field of an object.

    Attributes
    ----------
    apiVersion: str, default is v1, optional.
        A Container-level attribute.
        Version of the schema the FieldPath is written in terms of, defaults to "v1".
    fieldPath: str, default is Undefined, required.
        A Container-level attribute.
        Path of the field to select of an object.
    """

    apiVersion?: str = "v1"
    fieldPath: str

type Unit = units.NumberMultiplier
schema ResourceFieldSelector:
    """ ResourceFieldSelector represents container resources (cpu, memory) and their output format.

    Attributes
    ----------
    containerName: str, default is Undefined, required for volumes, optional for env vars.
        A Container-level attribute.
    resource: str, default is Undefined, required.
        A Container-level attribute.
        Resource to select.
    divisor: int | Unit, default is 1, optional.
        A Container-level attribute.
        Specifies the output format of the exposed resources, defaults to 1
    """
    containerName?: str
    resource: str
    divisor?: int | Unit = 1

schema EnvFromSource:
    """ EnvFromSource represents the source of a set of ConfigMaps or Secrets.

    Attributes
    ----------
    configMapRef: str, default is Undefined, optional.
        A Container-level attribute.
        The ConfigMap name to select from.
    secretRef: str, default is Undefined, optional.
        A Container-level attribute.
        The Secret name to select from.
    """

    configMapRef?: str
    secretRef?: str

    check:
        not configMapRef or not secretRef, "can not set configMapRef and secretRef in env at the same time"
        configMapRef or secretRef, "must specify one of: `configMapRef` or `secretRef`"
