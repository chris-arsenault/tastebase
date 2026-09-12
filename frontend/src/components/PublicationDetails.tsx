import type { PublicationRecommendation, SubscriptionOption } from "../types";
import { formatBookTagKey } from "../utils/bookTags";
import { MetaChip, StatusDot, type SignalState } from "./signals";

const outlookSignal: Record<
  NonNullable<PublicationRecommendation["outlook"]>["relationship"],
  SignalState
> = {
  aligned: "ok",
  compatible: "ok",
  neutral: "muted",
  contrast: "active",
  mixed: "info",
};

const outlookDetail: Record<
  NonNullable<PublicationRecommendation["outlook"]>["relationship"],
  string
> = {
  aligned: "Outlook aligned with mine",
  compatible: "Outlook compatible with mine",
  neutral: "Outlook neutral relative to mine",
  contrast: "Outlook contrasts with mine — a deliberate challenge",
  mixed: "Outlook mixed relative to mine",
};

function priceText(option: SubscriptionOption): string {
  if (!option.price) {
    return option.accessModel === "free" ? "Free access" : "Price not listed";
  }
  const base = `${option.price.amount} ${option.price.currency} / ${option.price.period}`;
  return option.price.priceType
    ? `${base} · ${formatBookTagKey(option.price.priceType)}`
    : base;
}

function Subscription({ option }: Readonly<{ option: SubscriptionOption }>) {
  return (
    <li className="publication-subscription">
      <div className="publication-subscription-heading">
        <strong>{option.label}</strong>
        {option.url && (
          <MetaChip label="Subscription page" href={option.url}>
            View option
          </MetaChip>
        )}
      </div>
      <p>
        {option.formats.join(" + ")} · {formatBookTagKey(option.accessModel)}
        {option.region && ` · ${option.region}`}
      </p>
      <p className="tabular">{priceText(option)}</p>
      <p className="publication-verified">
        {option.verifiedAt
          ? `Verified ${option.verifiedAt}`
          : "Verification date not recorded"}
      </p>
      {option.notes && <p>{option.notes}</p>}
    </li>
  );
}

function cadenceText(cadence: PublicationRecommendation["cadence"]): string {
  return cadence.issuesPerYear == null
    ? cadence.label
    : `${cadence.label} · ${cadence.issuesPerYear}/yr`;
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
      <div className="meta-row">
        <MetaChip label="Publication type">
          {formatBookTagKey(publication.publicationType)}
        </MetaChip>
        <MetaChip label="Cadence" icon={"📅"}>
          {cadenceText(publication.cadence)}
        </MetaChip>
        <MetaChip label="Audience level">
          {formatBookTagKey(publication.audienceLevel)}
        </MetaChip>
        {home && (
          <MetaChip label="Editorial home" icon={"📍"}>
            {home}
          </MetaChip>
        )}
        {publication.homepage && (
          <MetaChip label="Publication website" href={publication.homepage}>
            Website
          </MetaChip>
        )}
      </div>
      {publication.outlook && (
        <section className="publication-outlook">
          <div className="publication-outlook-heading">
            <h3>Outlook</h3>
            <StatusDot
              state={outlookSignal[publication.outlook.relationship]}
              text={formatBookTagKey(publication.outlook.relationship)}
              detail={outlookDetail[publication.outlook.relationship]}
            />
          </div>
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
