// Os seis operadores de comparação, entre int e entre bool
var a: int = 1;
var b: int = 2;
var t: bool = true;
print(a == b);
print(a != b);
print(a < b);
print(a > b);
print(a <= b);
print(a >= b);
print(t == false);
print(t != (a < b));
var c: bool = a + 1 <= b * 2;   // (a + 1) <= (b * 2)
print(c);
