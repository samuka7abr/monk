// while e if aninhados
var i: int = 0;
while (i < 3) {
    var j: int = 0;
    while (j < 2) {
        if (i == j) {
            print(i);
        } else {
            if (i > j) {
                print(i - j);
            }
        }
        j = j + 1;
    }
    i = i + 1;
}
