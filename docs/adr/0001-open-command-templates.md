# External tool invocations are open argv templates

bdar shells out to `xorrisofs`, `dvdisaster` and `cdrskin`. Rather than owning those command lines
and exposing a fixed set of options, every invocation is a user-editable argv array in the config
file, with `{placeholder}` substitution. bdar ships defaults compiled into the binary and treats the
config as a sparse overlay over them.

This is deliberate and it is the opposite of what the safety argument suggests, so it is worth
recording why.

## Considered options

**bdar owns the argv; config supplies a binary path and a few fields.** A prototype (the
`prototype/config-template-model` branch) rendered both designs side by side. On the happy path
they emit byte-identical commands, so the only thing open templates can do that owned argv cannot is
produce a *wrong* command. That looked decisive in favour of owning it.

It isn't, because the variance users need is real and is not enumerable: compression for corpora
unlike RAW photos, different symlink tradeoffs per dataset, an unusual burner, a different storage
medium entirely, a different version or implementation of any of the three tools, a richer readback
tool that reports more than pass/fail, and no-ops or spies for testing. Covering those with fields
means tracking the option surface of four complicated third-party programs — effectively
reimplementing their argument parsers, and still guessing wrong.

## Consequences

- **A new medium needs no bdar release.** A media profile carries arbitrary keys and may override
  any step's template, so writing to hardware nobody has thought of yet is a config change.
- **bdar cannot be the authority on tool behaviour.** Exit code is the contract. Only the sizing
  step's output is parsed. Everything else's stdout and stderr is captured verbatim for a human.
- **Validation is structural, not semantic.** Malformed config, an unknown step name or an
  unresolvable placeholder fail at load. The handful of known-destructive flag omissions produce an
  advisory warning by substring presence, silenceable per step. bdar does not parse arguments.
- **Correctness could not live in a template**, so it doesn't. Verifying a disc is bdar's own code
  and is not configurable — it is the one thing a user must not be able to weaken by editing a
  string.
- **Overridden steps can go stale.** bdar records the shipped default each override was forked from
  and warns when that specific default later changes, which is the one gap a sparse overlay leaves.
