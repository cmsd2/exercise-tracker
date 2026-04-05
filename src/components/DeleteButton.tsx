import { useState, useEffect, useRef } from "react";

interface DeleteButtonProps {
  onDelete: () => Promise<void> | void;
}

export function DeleteButton({ onDelete }: DeleteButtonProps) {
  const [state, setState] = useState<"idle" | "confirming">("idle");
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  function handleClick() {
    if (state === "confirming") {
      setState("idle");
      onDelete();
    } else {
      setState("confirming");
      timerRef.current = setTimeout(() => setState("idle"), 3000);
    }
  }

  const confirming = state === "confirming";

  return (
    <button
      className="btn"
      onClick={handleClick}
      style={{
        fontSize: "var(--font-size-xs)",
        padding: "2px 8px",
        background: confirming ? "#ff3b30" : "var(--color-bg-tertiary)",
        color: confirming ? "#fff" : "var(--color-danger)",
        fontWeight: confirming ? 600 : 500,
        transition: "background-color 0.25s, color 0.25s, transform 0.15s",
        transform: confirming ? "scale(1.05)" : "scale(1)",
      }}
    >
      {confirming ? "Confirm?" : "Delete"}
    </button>
  );
}
