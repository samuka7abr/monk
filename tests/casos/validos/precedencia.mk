// Precedência: unário antes de * e /, que vêm antes de + e -
var x: int = 1 + 2 * 3 - -4;    // 1 + (2 * 3) - (-4) = 11
var y: int = 10 - 6 / 2 * 3;    // 10 - ((6 / 2) * 3) = 1
var z: int = -x * y + 7;        // ((-x) * y) + 7 = -4
print(x);
print(y);
print(z);
