# Are you ready to write high quality code?

The idea of teenygrad is to build on the work George Hotz has done on tinygrad but in a library that can be used in high performance production environments. Where memory, speed and GPU/CPU usage are highly optimized. We aim to build support training/inference on a wide variety of ML models at 80%+ max theoretical speed across a huge variety of hardware.

There is almost no boilerplate code anywhere in this library, and you should help keep it that way. If the code you are contributing to core teenygrad, in `teenygrad/`, isn't some of the highest quality code you've written in your life, either put in the effort to make it great, or don't bother. (other directories have a slightly more relaxed standard)

There is a linter, but it's not complete. Spend a little time reading the existing code to get a feel for the style.

I love PRs where I can look at them and just say, yes, this will improve the codebase and click merge. If you have an incomplete PR, feel free to post it as a draft.

We value readability, however we also prioritize performance. Code which does not meet these criteria will be rejected.

There are a few basic ways to contribute:

## Bug-fixes

These are the most straightforward. Discover a bug. Add a test to reproduce it. Write a clean fix. Submit a PR. Confirm CI passes.

## Conceptual Cleanups

This is some of the highest value work in teenygrad. If you realize two 50 line functions are basically the same thing, and you can merge them, amazing! Things that look confusing and are hard to follow are probably poorly written. If you can rewrite the code and be like, oh that's a ton simpler, by all means do so. Make sure you have good test coverage around what you are changing.

## Better Testing

Always welcome! Think about how robust and fast your tests are though. How likely is this test to catch a bug? Tests that run in CI go in `tests/`.

## Features

This is a trickier one, as our goal is not to throw the kitchen sink into the core library. Therefore, novel transformers implementations or distributed mlops won't make it into the core library. However, we always welcome enabling technology into the core library.

 If there is a feature in PyTorch and numpy that you have actually seen people use, we probably want it. All new features must include good robust tests, and in general, matching the PyTorch API is good.
