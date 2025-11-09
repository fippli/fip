# Ansyncronous functions

definition

```
async long-process!: (x) {
  // some long running process
} // returns a promise

p: long-process!(123) // promise

result: await long-process!(234)
```
