fn main() {
    var x = 1u;
    let p = &x;
    var y = *p;
    *p = 2u;

    var arr = array<i32, 1>(1);
    let q = &arr[0];
    *q = 2;
}