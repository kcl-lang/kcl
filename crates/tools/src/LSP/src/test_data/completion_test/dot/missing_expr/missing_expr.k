schema ResourceRequirement:
    cpu?: str
    memory?: str

schema ResourceRequirements:
    limits?: ResourceRequirement
    requests?: ResourceRequirement

    check:
        limits?
