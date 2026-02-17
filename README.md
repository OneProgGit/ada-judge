# Ada Judge - a judgement system made with Rust

**To use it, you need nsjail to be installed!!!**

### Checker API:
```
(input: path, output: path, answer: path) -> Verdict
```

### Checker's verdicts:
```
0 - OK
1 - WrongAnswer
2 - PresentationError
3 - Fail
```

### Ada Judge's verdict is an enum:
```
Ok,
CompilationError,
RuntimeError,
TimeLimitExceeded,
MemoryLimitExceeded,
SecurityError,
WrongAnswer,
PresentationError,
Skipped,
```
