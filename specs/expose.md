---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: expose, reads, the read path
---
# expose

the fourth verb: answer reads over the committed graph. the read path, complement to validate/order/apply (the write path).

```
expose(query)  → rows | mutation batch     via [[inf]] (datalog) over committed state
expose(from,to) → attention                 a focus read, value from [[tru]]
```

cybergraph exposes its relations to [[inf]] and forwards focus reads to [[tru]]; it does not compute either. see [[query]] for the relation schema and [[attention]] for the focus read.

[verifiable private retrieval](../docs/private-retrieval.md) is a read-path
service profile for encrypted notifications and wallet recovery. cyber exposes
the capability through local or delegated workers; inf defines complete query
evaluation, bbg authenticates its domain, mudra supplies privacy operations, and
zheng proves the computation. the client verifies the requested scope and root.

mechanism, not decision: expose answers what it is asked. *which* questions to ask, and what to do with the answers, is the [[soma]] control loop — expose is the read primitive that loop calls.

see [[apply]] for the write counterpart · [[query]] for the interface · [[inf]] for the language · [[tru]] for focus.
