# Note on this result folder

This run was executed while the owner was running a heavy task on the same computer (the
machine was not idle, and Docker Desktop / the WSL virtual machine were active afterwards).

- **Accuracy figures** (WER, CER, critical flags, transcripts) do not depend on machine load: the
  engines are deterministic for a given input. They are expected to be reproduced identically by
  the clean re-run; any difference will be reported.
- **Speed, memory and busy-core figures** (RTF, inference time, cold load, peak memory) are NOT
  reliable from this folder. They were measured under contention and must not be quoted. Use the
  clean re-run instead.
- No quiet-machine check existed when this run was made; it was added right after (`bench run`
  now measures CPU use and power source before starting and refuses to run on a busy machine,
  unless `--force` is given, which is then recorded in `config.json`).

The folder is kept unchanged for transparency and for the determinism comparison.

## Update (clean re-run)

The re-run is `20261007-215116-full-owner-reps1-clean`. All 855 transcripts are identical to this folder, so the accuracy figures here are confirmed. The speed, memory and busy-core figures of the clean run (measured at about 20 % background CPU, see D-034 and the project log) replace the ones in this folder.
