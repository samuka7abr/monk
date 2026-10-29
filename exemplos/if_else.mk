// Saída esperada (uma por linha): 120, false, 2
var n: int = 5;
var f: int = 1;
while (n > 0) {
    f = f * n;
    n = n - 1;
}
if (f > 100) {
    print(f);
} else {
    print(0);
}
if (f < 100) {
    print(1);
} else {
    print(f == 0);
}
if (f < 10) {
    print(1);
} else if (f < 1000) {
    print(2);
} else {
    print(3);
}
