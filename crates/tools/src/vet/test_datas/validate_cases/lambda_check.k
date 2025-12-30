_check_uniq = lambda xs -> bool {
  isunique(xs)
}

schema LambdaCheck:
  elems: [int]

  check:
    _check_uniq(elems) if elems, "elems of ${elems} MUST be unique"
