import React from "react";
import { useAuth0 } from "@auth0/auth0-react";

export default function CheckoutButton({ onCheckout, theme }) {
  const { isAuthenticated, loginWithRedirect } = useAuth0();

  const handleClick = () => {
    if (!isAuthenticated) {
      loginWithRedirect({
        authorizationParams: {
          redirect_uri:
            process.env.REACT_APP_AUTH0_REDIRECT_URI || window.location.origin,
        },
      });
      return;
    }
    if (onCheckout) onCheckout();
  };

  return (
    <button
      type="button"
      onClick={handleClick}
      className="mt-4 w-full rounded-full px-5 py-3 text-sm font-semibold text-white"
      style={{ background: theme.ink }}
    >
      Checkout
    </button>
  );
}

