import type { ReactNode } from 'react';
export function Panel({ title, eyebrow, aside, children, className = '' }: { title: string; eyebrow?: string; aside?: ReactNode; children: ReactNode; className?: string }) {
  return <section className={`panel ${className}`}><div className="panel-heading"><div>{eyebrow && <p className="eyebrow">{eyebrow}</p>}<h2>{title}</h2></div>{aside}</div>{children}</section>;
}
export function Empty({ title, children }: { title: string; children: ReactNode }) {
  return <div className="empty"><span aria-hidden="true">◷</span><h3>{title}</h3><p>{children}</p></div>;
}
