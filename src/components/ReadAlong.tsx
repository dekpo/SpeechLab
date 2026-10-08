import { useEffect, useMemo, useRef, useState } from "react";
import { activeSegmentIndex, buildPieces, scrollTarget } from "../speech/readAlong";
import type { Segment } from "../speech/types";

/** After a manual scroll, automatic scrolling waits this long before it takes over again. */
const MANUAL_SCROLL_PAUSE_MS = 4000;

interface Props {
  /** The text the audio was generated from (never the normalised one). */
  text: string;
  segments: Segment[];
  /** The player the audio plays in; the view follows its clock and never controls it. */
  audio: HTMLAudioElement | null;
}

/**
 * Read-only view of the text with the sentence being read shaded. The highlight is driven by the
 * audio element's clock on every animation frame (the 4 Hz `timeupdate` event is too coarse), so
 * pause keeps it, seeking with the player's bar moves it and the page never starts playback itself.
 * Nothing is shaded before the first play, after "Stop" (paused at 0) or once the audio has ended.
 * Sentence segments keep their index, so a click-to-jump can be added later without changing the data.
 */
export default function ReadAlong({ text, segments, audio }: Props) {
  const [active, setActive] = useState(-1);
  const [follow, setFollow] = useState(true);
  const box = useRef<HTMLDivElement>(null);
  const manualUntil = useRef(0);
  // Set when the audio ends; cleared by the next play or seek.
  const finished = useRef(false);
  const pieces = useMemo(() => buildPieces(text, segments), [text, segments]);

  // Follow the player's clock.
  useEffect(() => {
    if (!audio) return;
    let frame = 0;
    const sync = () => {
      const idle = finished.current || (audio.paused && audio.currentTime === 0);
      setActive(idle ? -1 : activeSegmentIndex(segments, audio.currentTime * 1000));
    };
    const tick = () => {
      sync();
      frame = requestAnimationFrame(tick);
    };
    const start = () => {
      finished.current = false;
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(tick);
    };
    const halt = () => {
      cancelAnimationFrame(frame);
      sync();
    };
    const ended = () => {
      finished.current = true;
      cancelAnimationFrame(frame);
      setActive(-1);
    };
    const seeked = () => {
      finished.current = false;
      sync();
    };
    audio.addEventListener("play", start);
    audio.addEventListener("pause", halt);
    audio.addEventListener("seeked", seeked);
    audio.addEventListener("ended", ended);
    if (!audio.paused) start();
    else sync();
    return () => {
      cancelAnimationFrame(frame);
      audio.removeEventListener("play", start);
      audio.removeEventListener("pause", halt);
      audio.removeEventListener("seeked", seeked);
      audio.removeEventListener("ended", ended);
    };
  }, [audio, segments]);

  // A manual scroll (wheel, touch, scroll bar) suspends automatic scrolling for a few seconds.
  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const manual = () => {
      manualUntil.current = Date.now() + MANUAL_SCROLL_PAUSE_MS;
    };
    el.addEventListener("wheel", manual, { passive: true });
    el.addEventListener("touchmove", manual, { passive: true });
    el.addEventListener("pointerdown", manual);
    return () => {
      el.removeEventListener("wheel", manual);
      el.removeEventListener("touchmove", manual);
      el.removeEventListener("pointerdown", manual);
    };
  }, []);

  // Keep the active sentence in view.
  useEffect(() => {
    const el = box.current;
    if (!el || active < 0 || !follow || Date.now() < manualUntil.current) return;
    const span = el.querySelector<HTMLElement>(`[data-segment="${active}"]`);
    if (!span) return;
    const boxRect = el.getBoundingClientRect();
    const spanRect = span.getBoundingClientRect();
    const top = spanRect.top - boxRect.top + el.scrollTop;
    const target = scrollTarget(top, top + spanRect.height, el.scrollTop, el.clientHeight);
    if (target === null) return;
    const reduced = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    el.scrollTo({ top: target, behavior: reduced ? "auto" : "smooth" });
  }, [active, follow]);

  return (
    <div>
      <div className="readalong" ref={box} id="tts-readalong" tabIndex={0} aria-label="Text being read">
        {pieces.map((p, i) =>
          p.segment === null ? (
            <span key={i}>{p.text}</span>
          ) : (
            <span
              key={i}
              data-segment={p.segment}
              data-start-ms={segments[p.segment].startMs}
              data-end-ms={segments[p.segment].endMs}
              aria-current={p.segment === active ? "true" : undefined}
              className="sentence-span"
            >
              {p.text}
            </span>
          ),
        )}
      </div>
      <label className="check">
        <input id="tts-follow" type="checkbox" checked={follow} onChange={(e) => setFollow(e.target.checked)} />
        Follow reading (scroll automatically)
      </label>
    </div>
  );
}
