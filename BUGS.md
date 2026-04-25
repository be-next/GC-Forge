# Bugs

Living backlog of bugs found during implementation.
Maintained by the Tester-func role. See `doc/process/orchestration.md` for severity rules.

Format:

```
## BUG-NNN — <title>
- Severity: critical | high | medium | low
- Found: iter NN, YYYY-MM-DD
- Reporter: <role>
- Status: open | in-progress | fixed in iter MM | wont-fix
- Description, repro, expected vs observed.
```

Rules:

- `high` or `critical` → blocks DoD gate of the current iteration.
- `medium`/`low` → backlog, prioritized by Teamlead.
- Hard cap: > 15 open `medium` OR a `medium` stagnating > 3 iterations triggers automatic escalation.

---

_No bugs reported yet._
