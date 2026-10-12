import type { PublicationIssue } from "../types";
import { ClampText, MetaChip } from "./signals";

export function PublicationIssues({
  issues,
}: Readonly<{ issues: PublicationIssue[] }>) {
  if (issues.length === 0) return null;
  return (
    <details className="publication-issues">
      <summary>Issues ({issues.length})</summary>
      <ul>
        {issues.map((issue) => (
          <li key={issue.id} className="publication-issue">
            <h3>
              {issue.label}
              {issue.title && `: ${issue.title}`}
            </h3>
            <div className="meta-row">
              {issue.volume && (
                <MetaChip label="Volume">Vol. {issue.volume}</MetaChip>
              )}
              {issue.issueNumber && (
                <MetaChip label="Issue number">
                  No. {issue.issueNumber}
                </MetaChip>
              )}
              {issue.publicationDate && (
                <MetaChip label="Publication date">
                  {issue.publicationDate}
                </MetaChip>
              )}
              {issue.pageCount != null && (
                <MetaChip label="Page count">{issue.pageCount} pages</MetaChip>
              )}
              {issue.url && (
                <MetaChip label="Issue link" href={issue.url}>
                  View issue
                </MetaChip>
              )}
            </div>
            {issue.summary && <ClampText text={issue.summary} />}
            {issue.rating != null && issue.writeup.trim() && (
              <div className="book-saved-review">
                <div className="book-review-heading">
                  <h4>What I thought</h4>
                  <span
                    className="book-rating"
                    aria-label={`${issue.rating} out of 5 stars`}
                  >
                    {issue.rating}/5
                  </span>
                </div>
                <ClampText text={issue.writeup} />
              </div>
            )}
          </li>
        ))}
      </ul>
    </details>
  );
}
