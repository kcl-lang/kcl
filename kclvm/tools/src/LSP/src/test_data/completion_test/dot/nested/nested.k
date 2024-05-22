schema N:
    ab: int

schema ListN:
    a: [N]

list_N: ListN = {
    a: [{
        
    }]
}

schema DictN:
    a: {str:N}

dictN: DictN = {
    a.c = {
        
    }
}
dictN1: DictN = {
    a : {
        c: {
            
        }
    }
}
schema ListListN:
    a: [[N]]

listlistN: ListListN = {
    a: [[{
        
    }]]
}
