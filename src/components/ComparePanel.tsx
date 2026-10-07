import { useEffect, useRef, useState } from "react";
import { cancelTranscription, compareTexts, transcribe } from "../speech/api";
import type { ModelInfo, TextComparison, TranscribeResult } from "../speech/types";
import DiffView from "./DiffView";

interface Row {
  model: ModelInfo;
  status: "pending" | "running" | "done" | "error";
  result?: TranscribeResult;
  error?: string;
}

interface Props {
  models: ModelInfo[];
  language: string;
  audioPath: string;
}

const pct = (v: number | null) => (v === null ? "n/a" : `${(v * 100).toFixed(1)} %`);

export default function ComparePanel({ models, language, audioPath }: Props) {
  const installed = models.filter((m) => m.installStatus === "installed" && m.languages.includes(language));
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [rows, setRows] = useState<Row[]>([]);
  const [running, setRunning] = useState(false);
  const [reference, setReference] = useState("");
  const [baselineId, setBaselineId] = useState("");
  const [comparisons, setComparisons] = useState<Record<string, TextComparison>>({});
  const stop = useRef(false);

  // Drop selections that are no longer valid (language change, uninstalled model).
  useEffect(() => {
    setSelected((prev) => new Set([...prev].filter((id) => installed.some((m) => m.id === id))));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [language, models]);

  const done = rows.filter((r) => r.status === "done" && r.result);
  const effectiveBaseline = done.some((r) => r.model.id === baselineId) ? baselineId : done[0]?.model.id ?? "";
  const hasReference = reference.trim().length > 0;

  // Recompute WER/CER (against the reference) or the disagreement with the baseline row.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      const next: Record<string, TextComparison> = {};
      const baseText = done.find((r) => r.model.id === effectiveBaseline)?.result?.text ?? "";
      for (const r of done) {
        if (hasReference) next[r.model.id] = await compareTexts(reference, r.result!.text);
        else if (r.model.id !== effectiveBaseline) next[r.model.id] = await compareTexts(baseText, r.result!.text);
      }
      if (!cancelled) setComparisons(next);
    })().catch(() => setComparisons({}));
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rows, reference, effectiveBaseline]);

  const fastest = Math.min(...done.map((r) => r.result!.processingMs).filter((v) => v > 0), Infinity);

  async function run() {
    const chosen = installed.filter((m) => selected.has(m.id));
    if (chosen.length === 0 || !audioPath.trim()) return;
    stop.current = false;
    setRunning(true);
    setComparisons({});
    setRows(chosen.map((model) => ({ model, status: "pending" })));
    // Sequential on purpose: parallel runs would compete for the CPU and distort timings.
    for (const model of chosen) {
      if (stop.current) break;
      setRows((rs) => rs.map((r) => (r.model.id === model.id ? { ...r, status: "running" } : r)));
      try {
        const result = await transcribe({ providerId: model.provider, modelId: model.id, language, audioPath });
        setRows((rs) => rs.map((r) => (r.model.id === model.id ? { ...r, status: "done", result } : r)));
      } catch (e) {
        setRows((rs) => rs.map((r) => (r.model.id === model.id ? { ...r, status: "error", error: String(e) } : r)));
      }
    }
    setRunning(false);
  }

  function toggle(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <section>
      <strong>Engine comparison</strong>
      <p className="hint">
        Runs the selected audio through each chosen model one after the other and shows the results
        side by side. One recording says little: do not conclude that an engine is better from a
        single clip.
      </p>

      {installed.length === 0 && <p>No installed model supports "{language}".</p>}
      {installed.map((m) => (
        <label key={m.id} className="check">
          <input type="checkbox" checked={selected.has(m.id)} onChange={() => toggle(m.id)} disabled={running} />
          {m.displayName} <small>({m.provider})</small>
        </label>
      ))}

      <label htmlFor="ref">Reference transcript (optional, enables WER and CER)</label>
      <textarea id="ref" rows={2} value={reference} onChange={(e) => setReference(e.target.value)} />

      <button onClick={() => void run()} disabled={running || selected.size === 0 || !audioPath.trim()}>
        {running ? "Comparing…" : `Compare ${selected.size || ""} model(s)`}
      </button>
      <button
        onClick={() => {
          stop.current = true;
          void cancelTranscription();
        }}
        disabled={!running}
      >
        Stop
      </button>

      {rows.length > 0 && (
        <>
          <table>
            <thead>
              <tr>
                <th>Model</th><th>Decoding</th><th>Load</th><th>Inference</th><th>RTF</th>
                <th title="Inference time relative to the fastest model in this comparison">×fastest</th>
                {hasReference ? <><th>WER</th><th>CER</th></> : <th title="Word-level disagreement with the baseline text. NOT an error rate: the baseline can be wrong.">vs baseline</th>}
                <th>Baseline</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => {
                const c = comparisons[r.model.id];
                return (
                  <tr key={r.model.id}>
                    <td>{r.model.displayName}</td>
                    {r.status === "done" && r.result ? (
                      <>
                        <td>{r.result.decoding}</td>
                        <td>{r.result.loadMs} ms{r.result.coldStart ? "" : " (warm)"}</td>
                        <td>{r.result.processingMs} ms</td>
                        <td>{r.result.rtf?.toFixed(3) ?? "n/a"}</td>
                        <td>{Number.isFinite(fastest) && fastest > 0 ? (r.result.processingMs / fastest).toFixed(2) : "n/a"}</td>
                        {hasReference ? (
                          <><td>{pct(c?.wer ?? null)}</td><td>{pct(c?.cer ?? null)}</td></>
                        ) : (
                          <td>{r.model.id === effectiveBaseline ? "—" : pct(c?.wer ?? null)}</td>
                        )}
                        <td>
                          <input
                            type="radio"
                            name="baseline"
                            checked={effectiveBaseline === r.model.id}
                            onChange={() => setBaselineId(r.model.id)}
                            disabled={hasReference}
                            aria-label={`Use ${r.model.displayName} as baseline`}
                          />
                        </td>
                      </>
                    ) : (
                      <td colSpan={hasReference ? 8 : 7}>
                        {r.status === "error" ? <span className="errtext">{r.error}</span> : r.status}
                      </td>
                    )}
                  </tr>
                );
              })}
            </tbody>
          </table>

          {rows.map((r) =>
            r.status === "done" && r.result ? (
              <div key={r.model.id} className="resultblock">
                <strong>{r.model.displayName}</strong>
                {comparisons[r.model.id] ? (
                  <DiffView diff={comparisons[r.model.id].diff} />
                ) : (
                  <p>{r.result.text}</p>
                )}
              </div>
            ) : null,
          )}
          <p className="hint">
            Diff legend: <del>struck</del> = in the {hasReference ? "reference" : "baseline"} but not in this output,{" "}
            <ins>highlighted</ins> = in this output only. Scores use basic normalisation (case, punctuation,
            hyphens); digits and decimals are kept. Critical-error checks (numbers, units, negations) come in M5.
          </p>
        </>
      )}
    </section>
  );
}
