import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Auth0Provider } from '@auth0/auth0-react';
import './index.css';
import App from './App';
import AdminPanel from 'admin';
import reportWebVitals from './reportWebVitals';

const auth0Domain = process.env.REACT_APP_AUTH0_DOMAIN;
const auth0ClientId = process.env.REACT_APP_AUTH0_CLIENT_ID;
const auth0Audience = process.env.REACT_APP_AUTH0_AUDIENCE;
const auth0RedirectUri = process.env.REACT_APP_AUTH0_REDIRECT_URI || window.location.origin;

const routes = (
  <BrowserRouter>
    <Routes>
      <Route path="/imtheboss/*" element={<AdminPanel />} />
      <Route path="*" element={<App />} />
    </Routes>
  </BrowserRouter>
);

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    {auth0Domain && auth0ClientId ? (
      <Auth0Provider
        domain={auth0Domain}
        clientId={auth0ClientId}
        authorizationParams={{
          redirect_uri: auth0RedirectUri,
          ...(auth0Audience && { audience: auth0Audience }),
        }}
        cacheLocation="localstorage"
      >
        {routes}
      </Auth0Provider>
    ) : (
      routes
    )}
  </React.StrictMode>
);

reportWebVitals();
