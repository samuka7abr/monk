// Parênteses sobrepõem a precedência e podem aninhar
var a: int = (1 + 2) * 3;       // 9
var b: int = 2 * (3 + (4 - 1)); // 12
var c: int = -(a - b);          // 3
var d: bool = ((a) < (b));      // true
print(a);
print(b);
print(c);
print(d);
