import type { PublicationRecommendation, SubscriptionOption } from "../types";
import { formatBookTagKey } from "../utils/bookTags";

function Subscription({ option }: Readonly<{ option: SubscriptionOption }>) {
  return (
    <li className="publication-subscription">
      <strong>{option.label}</strong>
      <p>
        {option.formats.join(" + ")} · {formatBookTagKey(option.accessModel)}
        {option.region && ` · ${option.region}`}
      </p>
      {option.price ? (
        <p>
          {option.price.amount} {option.price.currency} / {option.price.period}
          {option.price.priceType &&
            ` · ${formatBookTagKey(option.price.priceType)}`}
        </p>
      ) : (
        <p>
          {option.accessModel === "free" ? "Free access" : "Price not listed"}
        </p>
      )}
      <p className="publication-verified">
        {option.verifiedAt
          ? `Verified ${option.verifiedAt}`
          : "Verification date not recorded"}
      </p>
      {option.notes && <p>{option.notes}</p>}
      {option.url && (
        <a href={option.url} target="_blank" rel="noreferrer">
          View option
        </a>
      )}
    </li>
  );
}

export function PublicationDetails({
  publication,
}: Readonly<{ publication: PublicationRecommendation }>) {
  const home = [
    publication.editorialHome?.city,
    publication.editorialHome?.country,
  ]
    .filter(Boolean)
    .join(", ");
  return (
    <div className="publication-details">
      <dl className="publication-facts">
        <div>
          <dt>Publication</dt>
          <dd>{formatBookTagKey(publication.publicationType)}</dd>
        </div>
        <div>
          <dt>Cadence</dt>
          <dd>
            {publication.cadence.label}
            {publication.cadence.issuesPerYear != null &&
              ` · ${publication.cadence.issuesPerYear} issues/year`}
          </dd>
        </div>
        <div>
          <dt>Audience</dt>
          <dd>{formatBookTagKey(publication.audienceLevel)}</dd>
        </div>
        {home && (
          <div>
            <dt>Editorial home</dt>
            <dd>{home}</dd>
          </div>
        )}
      </dl>
      {publication.homepage && (
        <a href={publication.homepage} target="_blank" rel="noreferrer">
          Publication website
        </a>
      )}
      {publication.outlook && (
        <section className="publication-outlook">
          <h3>
            Outlook · {formatBookTagKey(publication.outlook.relationship)}
          </h3>
          <p>{publication.outlook.summary}</p>
        </section>
      )}
      <details className="publication-subscriptions">
        <summary>
          Subscription options ({publication.subscriptions.length})
        </summary>
        {publication.subscriptions.length === 0 ? (
          <p>No subscription options recorded.</p>
        ) : (
          <ul>
            {publication.subscriptions.map((option, index) => (
              <Subscription key={`${option.label}:${index}`} option={option} />
            ))}
          </ul>
        )}
      </details>
    </div>
  );
}
