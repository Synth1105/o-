function memoize(fn) {
  const cache = {};
  return function(n) {
    if (n in cache) return cache[n];
    cache[n] = fn(n);
    return cache[n];
  };
}

const fib = memoize(function(n) {
  return n < 2 ? n : fib(n - 1) + fib(n - 2);
});

console.log(fib(50)); // 12586269025
