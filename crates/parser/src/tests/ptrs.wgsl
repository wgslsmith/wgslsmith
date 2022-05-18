var<private> global1: i32;
var<uniform> uniform1: i32;
var<storage> buf1: i32;
var<storage, read_write> buf2: i32;

fn main() {
    var x = 1u;
    let p = &x;
    var y = *p;
    *p = 2u;

    var arr = array<i32, 1>(1);
    let q = &arr[0];
    *q = 2;

    foo(q);
    foo(&arr[0]);

    // private
    {
        let p = &global1;
        *p = *p;
    }
    // uniform
    {
        let p = &uniform1;
        *p = *p;
    }
    // storage (read)
    {
        let p = &buf1;
        *p = *p;
    }
    // storage (read_write)
    {
        let p = &buf2;
        *p = *p;
    }
}

fn foo(p: ptr<private, i32>) {
    *p = 3;
}
