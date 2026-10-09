# Speech integration prompts (for AssistantCabinetAI)

> **STATUS: ON HOLD (D-050).** The owner will finish the Knowledge Base first, then relaunch a pass on the plan to adapt it.
> Authoritative status: `docs/integration/STATUS.md`. **Do not run `SP-1` to `SP-8` until it says `STATUS: RESUMED`.** What to
> run now, if anything: nothing, or `SP-R-rebaseline.md` when the Knowledge Base is finished, or `SP-0` if the owner explicitly wants
> the practice-PC probe earlier. The launchers below are draft v1 of 2026-10-09 and will be refreshed by the re-baseline pass.

Written 2026-10-09. One master brief and one launcher per phase. Plan: `docs/integration/03-integration-plan.md` (draft v1).

## Extra files in this folder

| File | Use |
|---|---|
| `SP-R-rebaseline.md` | **The next launcher once the Knowledge Base is finished**: tab "Speech R — Re-baseline the plan" |
| `KB-TAB-NOTE.md` | Drag into the Knowledge Base lot tabs (especially lots 2, 3, 8, 9, 11): tells those agents which contracts speech will rely on and to write a "Speech-relevant changes" line in their reports |
| `../../integration/KB-PREREQUISITES.md` | What speech needs from the KB, the conditions to resume, the facts to re-verify |
| `../../integration/STATUS.md` | The status board |

## How to use (one phase at a time)

1. Open a **new chat tab** with the right working folder (column "Working folder").
2. Name the tab with the name in the table.
3. **Drag** into it: `00-MASTER.md`, the phase launcher, and the documents the launcher lists. The launcher names them; they live in
   `C:\Users\elise\Documents\CURSOR\SpeechLab\docs\`.
4. The agent works, finishes green, writes its report, proposes a human test and gives you the git block. **You** run the git
   commands. The agent never commits.
5. Run the human test, say whether the phase is accepted, then start the next tab.

| Order | Tab name | Launcher | Working folder | Needs before |
|---|---|---|---|---|
| 0 | Speech 0 — Freeze and practice-PC probe | `SP-0-freeze-and-probe.md` | SpeechLab | M8 and integration documents committed |
| 1 | Speech 1 — Contracts and decisions | `SP-1-contracts.md` | AssistantCabinetAI | owner's answers Q-02, Q-03, Q-04 (`DECISIONS_TO_TAKE.md`) |
| 2 | Speech 2 — Port and whisper sidecar | `SP-2-port-and-sidecar.md` | AssistantCabinetAI | SP-1 accepted; the probe allows whisper.cpp |
| 3 | Speech 3 — Models and settings | `SP-3-models-and-settings.md` | AssistantCabinetAI | SP-2 merged; Q-05 |
| 4 | Speech 4 — Dictation | `SP-4-dictation.md` | AssistantCabinetAI | SP-3 merged; preferably after KB lot 7 or 8 |
| 5 | Speech 5 — Names and spelling | `SP-5-names-and-spelling.md` | AssistantCabinetAI | SP-4; KB lots 2, 3, 6, 9 merged |
| 6 | Speech 6 — Read aloud | `SP-6-read-aloud.md` | AssistantCabinetAI | SP-4; Q-06 |
| 7 | Speech 7 — Spoken conversation (optional) | `SP-7-conversation-mode.md` | AssistantCabinetAI | SP-4, SP-6; Q-07 yes |
| 8 | Speech 8 — Prove it | `SP-8-prove-it.md` | AssistantCabinetAI | the chosen phases accepted |

Phases 5 to 8 were written before the earlier phases ran. Each says so: the agent must re-check the repository before relying on
them. If a launcher is stale when you reach it, ask the agent of the previous tab (or a planning tab) to refresh it first.

## Why SP-0 is in the SpeechLab folder and the others are not

The AssistantCabinetAI rules say a handoff file is a local `docs/SESSION-<topic>.md` in that repository, dragged into the new chat.
SpeechLab's rules forbid modifying AssistantCabinetAI from here, and an agent in this repository must not write into that
one. These files therefore live in SpeechLab. If you prefer the AssistantCabinetAI convention, the SP-1 tab can copy them into
local `docs/SESSION-SPEECH-*.md` files (not versioned) as its first act; nothing else changes.

## Reminders that apply to every tab

Answer in French in the chat, write files in English. Git is yours only. No AI attribution in any commit, branch or PR text.
Everything local, nothing automatic: no cloud, no auto-send, no auto-play. macOS is on hold. Fictional fixtures only.
