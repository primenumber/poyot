fn[1] is_prime_impl(i, j) {
  if i < j*j {
    return 1;
  }
  if (i % j) == 0 {
    return 0;
  }
  return is_prime_impl(i, j+1);
}

fn[1] is_prime(i) {
  if i == 0 {
    return 0;
  } else {
    if i == 1 {
      return 0;
    }
  }
  return is_prime_impl(i, 2);
}

fn[0] primes(i, end) {
  if i > end {
    return;
  }
  if is_prime(i) {
    putchar('p');
  } else {
    putchar('c');
  }
  primes(i+1, end);
}

fn[0] main() {
  end = getnum();
  primes(1, end);
  putchar(10);
}
