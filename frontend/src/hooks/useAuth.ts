import {
  useEffect,
  useState,
  type Dispatch,
  type SetStateAction,
  type SubmitEvent,
} from "react";
import {
  getSession,
  signIn,
  signOut,
  type MfaSetupRedirect,
  type SoftwareTokenMfaChallenge,
  type SignInResult,
} from "../auth";
import type { CognitoUserSession } from "amazon-cognito-identity-js";

export type AuthState = {
  status:
    | "loading"
    | "signedOut"
    | "signingIn"
    | "signedIn"
    | "softwareTokenMfaRequired"
    | "mfaSetupRequired"
    | "verifyingMfa";
  token: string;
  username: string;
  mfaChallenge?: SoftwareTokenMfaChallenge;
  mfaSetupRedirect?: MfaSetupRedirect;
};

const signedOutAuth: AuthState = {
  status: "signedOut",
  token: "",
  username: "",
};

const getDisplayName = (session: CognitoUserSession, fallback: string) => {
  const payload = session.getIdToken().payload as Record<string, unknown>;
  return (
    (typeof payload.name === "string" && payload.name) ||
    (typeof payload.preferred_username === "string" &&
      payload.preferred_username) ||
    (typeof payload.email === "string" && payload.email) ||
    (typeof payload["cognito:username"] === "string" &&
      payload["cognito:username"]) ||
    fallback
  );
};

const signedInAuth = (
  session: CognitoUserSession,
  fallbackUsername: string,
): AuthState => ({
  status: "signedIn",
  token: session.getIdToken().getJwtToken(),
  username: getDisplayName(session, fallbackUsername),
});

const challengeAuth = (challenge: SoftwareTokenMfaChallenge): AuthState => ({
  status: "softwareTokenMfaRequired",
  token: "",
  username: challenge.username,
  mfaChallenge: challenge,
});

const mfaSetupAuth = (redirect: MfaSetupRedirect): AuthState => ({
  status: "mfaSetupRequired",
  token: "",
  username: redirect.username,
  mfaSetupRedirect: redirect,
});

const errorMessage = (error: unknown) => (error as Error).message;

type AuthSetter = Dispatch<SetStateAction<AuthState>>;
type MenuSetter = Dispatch<SetStateAction<boolean>>;
type AuthErrorHandler = (msg: string) => void;
type AuthSubmitHandler = (
  event: SubmitEvent<HTMLFormElement>,
  onError: AuthErrorHandler,
) => void;

const readFormValue = (form: HTMLFormElement, name: string) => {
  const value = new FormData(form).get(name);
  return typeof value === "string" ? value : "";
};

const finishSignIn = (
  setAuth: AuthSetter,
  setMenuOpen: MenuSetter,
  session: CognitoUserSession,
  username: string,
) => {
  setAuth(signedInAuth(session, username));
  setMenuOpen(false);
};

const applySignInResult = (
  setAuth: AuthSetter,
  setMenuOpen: MenuSetter,
  result: SignInResult,
) => {
  if (result.status === "signedIn") {
    finishSignIn(setAuth, setMenuOpen, result.session, result.username);
    return;
  }
  if (result.status === "softwareTokenMfaRequired") {
    setAuth(challengeAuth(result.challenge));
    return;
  }
  setAuth(mfaSetupAuth(result.redirect));
};

const createSignInHandler =
  (setAuth: AuthSetter, setMenuOpen: MenuSetter): AuthSubmitHandler =>
  (event, onError) => {
    event.preventDefault();
    const username = readFormValue(event.currentTarget, "username");
    const password = readFormValue(event.currentTarget, "password");
    onError("");
    setAuth({ status: "signingIn", token: "", username });
    signIn(username, password)
      .then((result) => applySignInResult(setAuth, setMenuOpen, result))
      .catch((error: unknown) => {
        setAuth(signedOutAuth);
        onError(errorMessage(error));
      });
  };

const createConfirmMfaHandler =
  (
    auth: AuthState,
    setAuth: AuthSetter,
    setMenuOpen: MenuSetter,
  ): AuthSubmitHandler =>
  (event, onError) => {
    event.preventDefault();
    if (!auth.mfaChallenge) {
      setAuth(signedOutAuth);
      onError("Start sign in again.");
      return;
    }
    const challenge = auth.mfaChallenge;
    const code = readFormValue(event.currentTarget, "mfaCode");
    onError("");
    setAuth({ ...auth, status: "verifyingMfa" });
    challenge
      .submitCode(code)
      .then((session) =>
        finishSignIn(setAuth, setMenuOpen, session, challenge.username),
      )
      .catch((error: unknown) => {
        setAuth(challengeAuth(challenge));
        onError(errorMessage(error));
      });
  };

const createSignOutHandler =
  (setAuth: AuthSetter, setMenuOpen: MenuSetter) => () => {
    signOut();
    setAuth(signedOutAuth);
    setMenuOpen(false);
  };

function useSessionBootstrap(setAuth: AuthSetter) {
  useEffect(() => {
    getSession()
      .then((session) => {
        if (session) {
          setAuth(signedInAuth(session, ""));
        } else {
          setAuth(signedOutAuth);
        }
      })
      .catch((error: unknown) => {
        console.error(error);
        setAuth(signedOutAuth);
      });
  }, [setAuth]);
}

function useOutsideMenuClose(menuOpen: boolean, setMenuOpen: MenuSetter) {
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (menuOpen && !target.closest(".menu-container")) setMenuOpen(false);
    };
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, [menuOpen, setMenuOpen]);
}

export function useAuth() {
  const [auth, setAuth] = useState<AuthState>({
    status: "loading",
    token: "",
    username: "",
  });
  const [menuOpen, setMenuOpen] = useState(false);
  useSessionBootstrap(setAuth);
  useOutsideMenuClose(menuOpen, setMenuOpen);
  const handleSignIn = createSignInHandler(setAuth, setMenuOpen);
  const handleConfirmMfa = createConfirmMfaHandler(auth, setAuth, setMenuOpen);
  const handleSignOut = createSignOutHandler(setAuth, setMenuOpen);

  return {
    auth,
    menuOpen,
    setMenuOpen,
    authActions: {
      signIn: handleSignIn,
      confirmMfa: handleConfirmMfa,
      signOut: handleSignOut,
    },
  };
}
