export function slugify(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

export function renderMarkdown(text: string): React.ReactNode[] {
  const unescaped = text.replace(/\\n/g, "\n");
  const paragraphs = unescaped.split(/\n\n/);
  return paragraphs.map((para, pi) => {
    const lines = para.split(/\n/);
    const children: React.ReactNode[] = [];
    lines.forEach((line, li) => {
      if (li > 0) children.push(<br key={`br-${pi}-${li}`} />);
      const parts = line.split(/(\*\*[^*]+\*\*)/g);
      parts.forEach((part, partIdx) => {
        const boldMatch = /^\*\*(.+)\*\*$/.exec(part);
        if (boldMatch) {
          children.push(
            <strong key={`b-${pi}-${li}-${partIdx}`}>{boldMatch[1]}</strong>,
          );
        } else {
          children.push(part);
        }
      });
    });
    return (
      <p key={`p-${pi}`} className="recipe-notes-paragraph">
        {children}
      </p>
    );
  });
}
