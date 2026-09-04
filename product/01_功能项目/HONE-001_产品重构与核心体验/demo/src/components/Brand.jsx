import { Link } from "react-router-dom";

export function Brand({ compact = false }) {
  return (
    <Link to="/" className="inline-flex items-center gap-2.5 text-foreground no-underline">
      <img src="/hone-logo.svg" alt="HONE" className="size-8 grayscale" />
      {!compact && <span className="text-[15px] font-semibold tracking-[0.2em]">HONE</span>}
    </Link>
  );
}
