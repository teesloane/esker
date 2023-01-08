---
title: foo
tags: elixir, bar
date_created: 2023-01-12
date_updated: 2023-01-12
---

Welcome to the future!

```js
function twentyfour(numbers, input) {
    var invalidChars = /[^\d\+\*\/\s-\(\)]/;

    var validNums = function(str) {
        // Create a duplicate of our input numbers, so that
        // both lists will be sorted.
        var mnums = numbers.slice();
        mnums.sort();

        // Sort after mapping to numbers, to make comparisons valid.
        return str.replace(/[^\d\s]/g, " ")
            .trim()
            .split(/\s+/)
            .map(function(n) { return parseInt(n, 10); })
            .sort()
            .every(function(v, i) { return v === mnums[i]; });
    };

    var validEval = function(input) {
        try {
            return eval(input);
        } catch (e) {
            return {error: e.toString()};
        }
    };

    if (input.trim() === "") return "You must enter a value.";
    if (input.match(invalidChars)) return "Invalid chars used, try again. Use only:\n + - * / ( )";
    if (!validNums(input)) return "Wrong numbers used, try again.";
    var calc = validEval(input);
    if (typeof calc !== 'number') return "That is not a valid input; please try again.";
    if (calc !== 24) return "Wrong answer: " + String(calc) + "; please try again.";
    return input + " == 24.  Congratulations!";
};

// I/O below.

while (true) {
    var numbers = [1, 2, 3, 4].map(function() {
        return Math.floor(Math.random() * 8 + 1);
    });

    var input = prompt(
        "Your numbers are:\n" + numbers.join(" ") +
        "\nEnter expression. (use only + - * / and parens).\n", +"'x' to exit.", "");

    if (input === 'x') {
        break;
    }
    alert(twentyfour(numbers, input));
}
```



Nulla volutpat mauris metus, eu elementum ex iaculis auctor. Mauris pellentesque augue a eros laoreet, quis tempus tortor tristique. Maecenas congue enim felis, eget condimentum turpis eleifend et. Nam condimentum faucibus lectus, eget ornare libero tristique ac. Maecenas id ullamcorper quam. Cras finibus orci volutpat congue dignissim. Quisque eu risus iaculis, lobortis nibh eu, mattis nibh. Vivamus ultricies euismod erat, ut imperdiet erat gravida non. Curabitur elementum bibendum nibh quis tempor. Donec porta ultrices velit in malesuada.

Morbi lacinia, diam vitae venenatis euismod, mauris justo sollicitudin purus, sed feugiat nulla erat et ante. Suspendisse elementum risus quis urna consectetur, eget rhoncus lorem pretium. Phasellus interdum ultricies lectus, egestas vestibulum leo porttitor nec. Duis facilisis, neque nec venenatis varius, nisl purus mattis urna, nec hendrerit ligula leo vitae erat. Phasellus ac turpis malesuada, blandit massa eu, molestie dolor. Quisque id faucibus ligula. Vestibulum orci sapien, ultrices vitae eleifend non, fermentum sit amet libero. 
