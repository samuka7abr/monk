// Menos unário empilhado: --x é - (- x)
var x: int = 5;
var y: int = --x;               // 5
var z: int = - -x;              // 5
var w: int = 1 - -x;            // 6
var v: int = ---x * 2;          // -10
print(y);
print(z);
print(w);
print(v);
