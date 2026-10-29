// Escopos aninhados e sombreamento
var x: int = 1;
print(x);                   // 1
{
    var x: int = 10;
    print(x);               // 10
    {
        var x: bool = true;
        print(x);           // true
    }
    x = x + 5;
    print(x);               // 15
    {
        var x: int = x + 1; // o x da direita é o do bloco do meio
        print(x);           // 16
    }
    print(x);               // 15
}
print(x);                   // 1
