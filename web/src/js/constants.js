export const SHARE_QUERY_KEY = 's';

export const SNIPPETS = [
  {
    label: 'Hello World!',
    value: `"Hello" + " " + "World!"`,
  },

  {
    label: 'Computing integers',
    value: `(10 + 2) * 30 + 5`,
  },

  {
    label: 'Debugging values',
    value: `puts("Output values");
puts(true, false, 100);
puts("You can clear the console with 'Ctrl-L' key !!");`,
  },

  {
    label: 'Conditionals',
    value: `if (true) {
  puts("Hello");
} else {
  puts("unreachable");
}`,
  },

  {
    label: 'Variable bindings',
    value: `let x = 10;
let y = 15;

x + y;`,
  },

  {
    label: 'Array',
    value: `let arr = ["one", "two", "three"];

puts(arr[0]);
puts(arr[1]);
puts(arr[2]);

puts("---- >8 ----");

puts(len(arr));
puts(first(arr));
puts(last(arr));

puts("---- >8 ----");

let arr = push(arr, "four");
puts(last(arr));`,
  },

  {
    label: 'Hashes',
    value: `let hash = { "name": "Jimmy", "age": 72, "band": "Led Zeppelin" };
puts(hash["name"]);
puts(hash["band"]);
puts(hash["age"]);

puts("---- >8 ----");

let hash = { true: "yes, a boolean", 99: "correct, an integer" };
puts(hash[5 > 1]);
puts(hash[100 - 1]);
`,
  },

  {
    label: 'Function',
    value: `let factorial = fn(n) {
  if (n == 0) {
    1
  } else {
    n * factorial(n - 1)
  }
}

factorial(5)`,
  },

  {
    label: 'Closures',
    value: `let newAdder = fn(x) {
  fn(y) { x + y };
};

let addTwo = newAdder(2);
addTwo(2);`,
  },

  {
    label: 'Array - Map',
    value: `let map = fn(arr, f) {
  let iter = fn(arr, accumulated) {
    if (len(arr) == 0) {
      accumulated
    } else {
      iter(rest(arr), push(accumulated, f(first(arr))))
    }
  }

  iter(arr, []);
};

let arr = [1, 2, 3, 4];

let double = fn(x) { x * 2 };

map(arr, double);`,
  },

  {
    label: 'Array - Reduce',
    value: `let reduce = fn(arr, initial, f) {
  let iter = fn(arr, result) {
    if (len(arr) == 0) {
      result
    } else {
      iter(rest(arr), f(result, first(arr)))
    }
  }

  iter(arr, initial);
};

let arr = [1, 2, 3, 4];

let sum = fn(arr) {
  reduce(arr, 0, fn(initial, el) { initial + el })
};

sum(arr);`,
  },
];
