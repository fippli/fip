# Core Identity

Identity helpers preserve or check values without transforming them. They are useful in pipelines, conditional guards, and when bridging impure and pure code. Review [functions](../functions.md) for details on currying and invocation syntax.

## identity

**Signature** `identity: (x) -> x`

**Behavior** Returns the provided argument unchanged. Often used as a default callback or to reset a pipeline.

**Example**

```fip
identity("hello")
// -> "hello"
```
