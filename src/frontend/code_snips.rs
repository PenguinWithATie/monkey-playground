pub const FIB_CODE: &str = "let fib = fn(n) {
    if (n < 2) {
        return 1;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
};
puts(fib(25));
";

pub const FIZZBUZZ_CODE: &str = "let fizzbuzz = fn(n) {
    if (n == 1) {
        puts(1);
        return 0;
    }

    if (n % 15 == 0) {
        puts(\"fizzbuzz\");
        return fizzbuzz(n-1);
    }
    if (n % 5 == 0) {
        puts(\"buzz\");
        return fizzbuzz(n-1);
    }
    if (n % 3 == 0) {
        puts(\"fizz\");
        return fizzbuzz(n-1);
    }

    puts(n);
    fizzbuzz(n-1);
};

fizzbuzz(100);";

pub const DOUBLE_W_MAP: &str = "let map = fn(arr, f) {
    let iter = fn (arr, accumulated) {
        if (len(arr) == 0) {
            accumulated
        } else {
            iter(rest(arr), push(accumulated, f(first(arr))));
        }
    };
    iter(arr, []);
};

let a = [1, 2, 3, 4];
let double = fn(x) { x * 2 };

puts(\"Before double: \", a);
puts(\"After double: \", map(a, double));";
