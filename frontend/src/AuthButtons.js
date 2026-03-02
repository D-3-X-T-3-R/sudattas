/**
 * Log in / Log out and account for Auth0. Only render when Auth0 is configured (inside Auth0Provider).
 */
import { useAuth0 } from "@auth0/auth0-react";
import { clearAccessToken } from "./authStore";

const auth0RedirectUri = process.env.REACT_APP_AUTH0_REDIRECT_URI || window.location.origin;

export default function AuthButtons() {
  const { isAuthenticated, user, loginWithRedirect, logout } = useAuth0();

  const handleLogin = () => {
    loginWithRedirect({
      authorizationParams: {
        redirect_uri: auth0RedirectUri,
        audience: process.env.REACT_APP_AUTH0_AUDIENCE || undefined,
      },
    });
  };

  const handleLogout = () => {
    clearAccessToken();
    logout({
      logoutParams: {
        returnTo: auth0RedirectUri,
      },
    });
  };

  if (isAuthenticated) {
    return (
      <div className="flex items-center gap-2">
        <span className="hidden text-sm text-[#6B7280] sm:inline" title={user?.email}>
          {user?.name || user?.email || "Account"}
        </span>
        <button
          type="button"
          onClick={handleLogout}
          className="rounded-full border px-3 py-1.5 text-xs font-medium hover:bg-white"
          style={{ borderColor: "var(--theme-line, #e5e7eb)" }}
        >
          Log out
        </button>
      </div>
    );
  }

  return (
    <button
      type="button"
      onClick={handleLogin}
      className="rounded-full border px-3 py-1.5 text-xs font-medium hover:bg-white"
      style={{ borderColor: "var(--theme-line, #e5e7eb)" }}
    >
      Log in
    </button>
  );
}
