# Advisories

This section tracks dependency and security advisory decisions.

## Contents

- [Dependencies](dependencies.md)
- [Accepted](accepted.md)
- [Resolved](resolved.md)

## Triage Workflow

```mermaid
flowchart TD
    Found["advisory found"] --> Reachable{"reachable?"}
    Reachable -- no --> Accept["document in accepted.md"]
    Reachable -- yes --> Fixable{"fix available?"}
    Fixable -- yes --> Patch["upgrade or patch"]
    Patch --> Verify["test and rescan"]
    Verify --> Resolve["document in resolved.md"]
    Fixable -- no --> Mitigate["mitigate or pin"]
    Mitigate --> Accept
```
