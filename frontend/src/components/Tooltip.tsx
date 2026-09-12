import { useEffect, useId, useRef, useState, type ReactNode } from "react";

type Align = "center" | "start" | "end";

function measureAlign(root: HTMLElement, bubble: HTMLElement): Align {
  const anchor = root.getBoundingClientRect();
  const width = bubble.offsetWidth;
  const center = anchor.left + anchor.width / 2;
  if (center - width / 2 < 8) return "start";
  if (center + width / 2 > window.innerWidth - 8) return "end";
  return "center";
}

/**
 * Tooltip that opens on mouse hover, keyboard focus within, and tap.
 *
 * The bubble is always in the DOM and linked via aria-describedby, so the
 * text is never hover-only for assistive tech. Listeners are attached
 * natively so the wrapper stays a plain, non-interactive span; the child
 * decides whether it is focusable. Near a viewport edge the bubble
 * re-aligns so it stays on screen.
 */
export function Tooltip({
  label,
  children,
  className = "",
}: Readonly<{
  label: ReactNode;
  children: ReactNode;
  className?: string;
}>) {
  const id = useId();
  const rootRef = useRef<HTMLSpanElement>(null);
  const bubbleRef = useRef<HTMLSpanElement>(null);
  const [open, setOpen] = useState(false);
  const [align, setAlign] = useState<Align>("center");

  useEffect(() => {
    const root = rootRef.current;
    const bubble = bubbleRef.current;
    if (!root || !bubble) return;
    const show = () => {
      setAlign(measureAlign(root, bubble));
      setOpen(true);
    };
    const hide = () => setOpen(false);
    const toggle = () => {
      setAlign(measureAlign(root, bubble));
      setOpen((current) => !current);
    };
    const onEnter = (event: PointerEvent) => {
      if (event.pointerType === "mouse") show();
    };
    const onLeave = (event: PointerEvent) => {
      if (event.pointerType === "mouse") hide();
    };
    root.addEventListener("pointerenter", onEnter);
    root.addEventListener("pointerleave", onLeave);
    root.addEventListener("focusin", show);
    root.addEventListener("focusout", hide);
    root.addEventListener("click", toggle);
    return () => {
      root.removeEventListener("pointerenter", onEnter);
      root.removeEventListener("pointerleave", onLeave);
      root.removeEventListener("focusin", show);
      root.removeEventListener("focusout", hide);
      root.removeEventListener("click", toggle);
    };
  }, []);

  useEffect(() => {
    if (!open) return;
    const closeOutside = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", closeOutside);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", closeOutside);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <span
      ref={rootRef}
      className={`tip ${open ? "tip-open" : ""} tip-${align} ${className}`}
      aria-describedby={id}
    >
      {children}
      <span ref={bubbleRef} id={id} role="tooltip" className="tip-bubble">
        {label}
      </span>
    </span>
  );
}
