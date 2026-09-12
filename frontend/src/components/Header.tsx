import { useCallback, type SubmitEvent } from "react";
import type { AuthState } from "../hooks/useAuth";
import type { MfaSetupRedirect, SoftwareTokenMfaChallenge } from "../auth";
import type { AppSection, Filters, ProductType } from "../types";

type AuthSubmitHandler = (
  event: SubmitEvent<HTMLFormElement>,
  onError: (msg: string) => void,
) => void;
type AuthActions = {
  signIn: AuthSubmitHandler;
  confirmMfa: AuthSubmitHandler;
  signOut: () => void;
};

const brandTaglines: Record<string, string> = {
  drink: "Drink Log",
  sauce: "Sauce Log",
  all: "Culinary Log",
};

function SignedInMenu({
  username,
  onSignOut,
}: Readonly<{
  username: string;
  onSignOut: () => void;
}>) {
  return (
    <>
      <div className="menu-user">
        <span className="menu-user-label">Signed in as</span>
        <span className="menu-user-name">{username || "Taster"}</span>
      </div>
      <button className="menu-item" onClick={onSignOut}>
        Sign out
      </button>
    </>
  );
}

function SignInForm({
  onSignIn,
  onError,
}: Readonly<{
  onSignIn: AuthSubmitHandler;
  onError: (msg: string) => void;
}>) {
  return (
    <form className="menu-auth-form" onSubmit={(e) => onSignIn(e, onError)}>
      <input
        name="username"
        placeholder="Username"
        required
        autoComplete="username"
      />
      <input
        name="password"
        type="password"
        placeholder="Password"
        required
        autoComplete="current-password"
      />
      <button type="submit" className="btn-primary">
        Sign in
      </button>
    </form>
  );
}

function MfaCodeForm({
  challenge,
  onConfirmMfa,
  onSignOut,
  onError,
}: Readonly<{
  challenge: SoftwareTokenMfaChallenge;
  onConfirmMfa: AuthSubmitHandler;
  onSignOut: () => void;
  onError: (msg: string) => void;
}>) {
  return (
    <form className="menu-auth-form" onSubmit={(e) => onConfirmMfa(e, onError)}>
      <span className="menu-auth-title">{challenge.username}</span>
      <input
        name="mfaCode"
        placeholder="6-digit code"
        required
        inputMode="numeric"
        autoComplete="one-time-code"
        maxLength={6}
      />
      <button type="submit" className="btn-primary">
        Verify
      </button>
      <button type="button" className="btn-ghost" onClick={onSignOut}>
        Start over
      </button>
    </form>
  );
}

function MfaSetupRedirectPanel({
  redirect,
  onSignOut,
}: Readonly<{
  redirect: MfaSetupRedirect;
  onSignOut: () => void;
}>) {
  return (
    <div className="menu-auth-form">
      <span className="menu-auth-title">MFA setup required</span>
      <span className="menu-auth-note">
        Enroll an authenticator in Ahara Business, then return to sign in.
      </span>
      <a className="menu-auth-setup-link" href={redirect.enrollmentUrl}>
        Open Ahara Business
      </a>
      <button type="button" className="btn-ghost" onClick={onSignOut}>
        Return to sign in
      </button>
    </div>
  );
}

function AuthMenu({
  auth,
  authActions,
  onError,
}: Readonly<{
  auth: AuthState;
  authActions: AuthActions;
  onError: (msg: string) => void;
}>) {
  if (auth.status === "signedIn") {
    return (
      <SignedInMenu username={auth.username} onSignOut={authActions.signOut} />
    );
  }
  if (auth.status === "signedOut") {
    return <SignInForm onSignIn={authActions.signIn} onError={onError} />;
  }
  if (auth.mfaChallenge?.kind === "softwareTokenMfa") {
    return (
      <MfaCodeForm
        challenge={auth.mfaChallenge}
        onConfirmMfa={authActions.confirmMfa}
        onSignOut={authActions.signOut}
        onError={onError}
      />
    );
  }
  if (auth.mfaSetupRedirect) {
    return (
      <MfaSetupRedirectPanel
        redirect={auth.mfaSetupRedirect}
        onSignOut={authActions.signOut}
      />
    );
  }
  return <div className="menu-loading">Loading...</div>;
}

const sections: { id: AppSection; label: string }[] = [
  { id: "tastings", label: "Tastings" },
  { id: "recipes", label: "Recipes" },
  { id: "books", label: "Bookshelf" },
];

function SectionToggle({
  section,
  onSectionChange,
}: Readonly<{
  section: AppSection;
  onSectionChange: (section: AppSection) => void;
}>) {
  return (
    <div
      className="segmented segmented-accent"
      role="group"
      aria-label="Section"
    >
      {sections.map((entry) => (
        <button
          key={entry.id}
          type="button"
          className={section === entry.id ? "active" : ""}
          aria-pressed={section === entry.id}
          onClick={() => onSectionChange(entry.id)}
        >
          {entry.label}
        </button>
      ))}
    </div>
  );
}

const productTypes: {
  id: ProductType | "all";
  label: string;
  title: string;
}[] = [
  { id: "sauce", label: "Sauces", title: "Hot sauces" },
  { id: "all", label: "All", title: "All items" },
  { id: "drink", label: "Drinks", title: "Drinks" },
];

function ProductToggle({
  productType,
  setProductType,
}: Readonly<{
  productType: string;
  setProductType: (pt: ProductType | "all") => void;
}>) {
  return (
    <div className="segmented" role="group" aria-label="Product type">
      {productTypes.map((entry) => (
        <button
          key={entry.id}
          type="button"
          className={productType === entry.id ? "active" : ""}
          aria-pressed={productType === entry.id}
          onClick={() => setProductType(entry.id)}
          title={entry.title}
        >
          {entry.label}
        </button>
      ))}
    </div>
  );
}

type MenuState = { open: boolean; setOpen: (open: boolean) => void };
type RefreshState = { refreshing: boolean; onRefresh: () => void };

function HeaderActions({
  auth,
  section,
  formOpen,
  refresh,
  menu,
  onAdd,
  onCloseForm,
  authActions,
  onError,
}: Readonly<{
  auth: AuthState;
  section: AppSection;
  formOpen: boolean;
  refresh: RefreshState;
  menu: MenuState;
  onAdd: () => void;
  onCloseForm: () => void;
  authActions: AuthActions;
  onError: (msg: string) => void;
}>) {
  const signedIn = auth.status === "signedIn";
  return (
    <div className="header-actions">
      {signedIn && (
        <button
          className="refresh-btn btn-icon"
          onClick={refresh.onRefresh}
          disabled={refresh.refreshing}
          title="Refresh data"
          aria-label="Refresh data"
        >
          {refresh.refreshing ? "..." : "↻"}
        </button>
      )}

      {signedIn && section === "tastings" && (
        <button
          className="add-btn btn-icon"
          onClick={() => (formOpen ? onCloseForm() : onAdd())}
          title={formOpen ? "Close" : "Add tasting"}
          aria-label={formOpen ? "Close form" : "Add tasting"}
        >
          {formOpen ? "×" : "+"}
        </button>
      )}

      <div className="menu-container">
        <button
          className="menu-btn btn-icon"
          onClick={(e) => {
            e.stopPropagation();
            menu.setOpen(!menu.open);
          }}
          aria-label="Menu"
          aria-expanded={menu.open}
        >
          <span className="menu-icon" />
        </button>
        {menu.open && (
          <div className="menu-dropdown">
            <AuthMenu auth={auth} authActions={authActions} onError={onError} />
          </div>
        )}
      </div>
    </div>
  );
}

type HeaderProps = {
  auth: AuthState;
  filters: Filters;
  setFilters: React.Dispatch<React.SetStateAction<Filters>>;
  section: AppSection;
  onSectionChange: (section: AppSection) => void;
  formOpen: boolean;
  refresh: RefreshState;
  menu: MenuState;
  onAdd: () => void;
  onCloseForm: () => void;
  authActions: AuthActions;
  onError: (msg: string) => void;
};

export function Header({
  auth,
  filters,
  setFilters,
  section,
  onSectionChange,
  formOpen,
  refresh,
  menu,
  onAdd,
  onCloseForm,
  authActions,
  onError,
}: Readonly<HeaderProps>) {
  const setProductType = useCallback(
    (pt: ProductType | "all") => setFilters((f) => ({ ...f, productType: pt })),
    [setFilters],
  );

  return (
    <header className="header">
      <div className="header-brand">
        <h1>Tastebase</h1>
        <span className="header-tagline">
          {section === "books"
            ? "Reading Log"
            : (brandTaglines[filters.productType] ?? "Culinary Log")}
        </span>
      </div>

      <div className="header-nav">
        <SectionToggle section={section} onSectionChange={onSectionChange} />
        {section === "tastings" && (
          <ProductToggle
            productType={filters.productType}
            setProductType={setProductType}
          />
        )}
      </div>

      <HeaderActions
        auth={auth}
        section={section}
        formOpen={formOpen}
        refresh={refresh}
        menu={menu}
        onAdd={onAdd}
        onCloseForm={onCloseForm}
        authActions={authActions}
        onError={onError}
      />
    </header>
  );
}
