fn[1] fib(x) {
  if x == 0 {
    return 0;
  } else {
    if x == 1 {
      return 1;
    } else {
      return fib(x-1) + fib(x-2);
    }
  }
}

fn[0] main() {
  x = getnum();
  putnum(fib(x));
  putchar(10);
}
