schema Data:
    mixin [DataMixin]
    labels: {str:} = {}
    metaLabels: {str:} = labels
        
schema DataMixin:
    labels = {
        "key": "value"  
    }
        
data = Data {}
