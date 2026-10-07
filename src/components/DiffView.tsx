import type { DiffOp } from "../speech/types";

/** Word-level diff: removed/changed reference words struck through, hypothesis words highlighted. */
export default function DiffView({ diff }: { diff: DiffOp[] }) {
  return (
    <p className="diff">
      {diff.map((op, i) => {
        switch (op.kind) {
          case "equal":
            return <span key={i}>{op.hypothesis} </span>;
          case "substitute":
            return (
              <span key={i}>
                <del>{op.reference}</del> <ins>{op.hypothesis}</ins>{" "}
              </span>
            );
          case "delete":
            return (
              <span key={i}>
                <del>{op.reference}</del>{" "}
              </span>
            );
          case "insert":
            return (
              <span key={i}>
                <ins>{op.hypothesis}</ins>{" "}
              </span>
            );
        }
      })}
    </p>
  );
}
