fn[0] hanoi_impl(n, src, dest) {
  if n == 0 {
    return;
  }
  mid = 6 - (src + dest);
  hanoi_impl(n-1, src, mid);
  putnum(n);
  putchar(',');
  putnum(src);
  putchar(',');
  putnum(dest);
  putchar(10);
  hanoi_impl(n-1, mid, dest);
}

fn[0] hanoi(n) {
  hanoi_impl(n, 1, 3);
}

fn[0] main() {
  hanoi(getnum());
}
