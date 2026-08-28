import { PublicClientApplication } from '@azure/msal-browser';

const clientId = import.meta.env.VITE_ENTRA_CLIENT_ID || '25c704f4-465a-47af-80ab-2c489466b697';
const tenant = import.meta.env.VITE_ENTRA_TENANT_ID || '35c6fe40-0ec0-46b6-98c6-213ad4de6650';
const subdomain = import.meta.env.VITE_ENTRA_TENANT_SUBDOMAIN || 'sociobotcustomers';
const authority = `https://${subdomain}.ciamlogin.com/${tenant}`;

export const ciam = new PublicClientApplication({
  auth: { clientId, authority, redirectUri: `${window.location.origin}/auth/callback`, knownAuthorities: [`${subdomain}.ciamlogin.com`] },
  cache: { cacheLocation: 'sessionStorage' },
});

export async function startSignIn(): Promise<void> {
  await ciam.initialize();
  await ciam.loginRedirect({ scopes: ['openid', 'profile', 'email'] });
}

export async function finishSignIn(): Promise<string | null> {
  await ciam.initialize();
  await ciam.handleRedirectPromise();
  const account = ciam.getActiveAccount() ?? ciam.getAllAccounts()[0];
  if (!account) return null;
  ciam.setActiveAccount(account);
  const token = await ciam.acquireTokenSilent({ account, scopes: ['openid', 'profile', 'email'] });
  return token.accessToken || token.idToken;
}

export async function signOut(): Promise<void> {
  await ciam.initialize();
  const account = ciam.getActiveAccount();
  await ciam.logoutRedirect({ account, postLogoutRedirectUri: window.location.origin });
}
