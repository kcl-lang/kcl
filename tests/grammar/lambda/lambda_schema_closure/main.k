# Test lambda closure capture in schemas
# Ensure that lambdas don't capture local variables from previous executions

schema Counter:
    count: int = 0

    add: (int) -> Counter = lambda n: int {
        Counter{count: count + n}
    }

c1 = Counter{}
c2 = c1.add(1)
c3 = c2.add(1)
c4 = c3.add(10)
c5 = c4.add(20)

# Verify that each call gets the correct count value
