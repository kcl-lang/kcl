# Test for issue #2027: Scientific notation should be treated as string when type=str

# Test case 1: 12e1 should be treated as string "12e1", not number 120
imageSHA = option("imageSHA", type="str")
imageSHA2 = imageSHA + '2'

# Test case 2: 1.5e-3 should be treated as string "1.5e-3"
val1 = option("val1", type="str")

# Test case 3: 2E10 should be treated as string "2E10"
val2 = option("val2", type="str")

# Test case 4: Regular numbers should still work as numbers
count = option("count", type="int")
count_doubled = count * 2

# Test case 5: Scientific notation without explicit type should work
sciVal = option("sciVal", type="str")
