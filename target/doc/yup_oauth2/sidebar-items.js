window.SIDEBAR_ITEMS = {"enum":[["Error","Encapsulates all possible results of the `token(...)` operation"],["InstalledFlowReturnMethod","Method by which the user agent return token to this application."]],"fn":[["parse_application_secret","Read an application secret from a JSON string."],["parse_service_account_key","Read a service account key from a JSON string."],["read_application_secret","Read an application secret from a file."],["read_authorized_user_secret","Read an authorized user secret from a JSON file. You can obtain it by running on the client: `gcloud auth application-default login`. The file should be on Windows in: `%APPDATA%/gcloud/application_default_credentials.json` for other systems: `$HOME/.config/gcloud/application_default_credentials.json`."],["read_service_account_key","Read a service account key from a JSON file. You can download the JSON keys from the Google Cloud Console or the respective console of your service provider."]],"mod":[["access_token","pseudo authenticator for use with plain access tokens. If you use a specialized service to manage your OAuth2-tokens you may get just the fresh generated access token from your service. The intention behind this is that if two services using the same refresh token then each service will invalitate the access token of the other service by generating a new token."],["authenticator","Module containing the core functionality for OAuth2 Authentication."],["authenticator_delegate","Module containing types related to delegates."],["authorized_user","This module provides a token source (`GetToken`) that obtains tokens using user credentials for use by software (i.e., non-human actors) to get access to Google services."],["error","Module containing various error types."],["service_account_impersonator","This module provides an authenticator that uses authorized user secrets to generate impersonated service account tokens."],["storage","Interface for storing tokens so that they can be re-used. There are built-in memory and file-based storage providers. You can implement your own by implementing the TokenStorage trait."]],"struct":[["AccessToken","Represents a token returned by oauth2 servers. All tokens are Bearer tokens. Other types of tokens are not supported."],["AccessTokenAuthenticator","Create a access token authenticator for use with pre-generated access tokens"],["ApplicationDefaultCredentialsAuthenticator","Create an authenticator that uses a application default credentials."],["ApplicationDefaultCredentialsFlowOpts","Provide options for the Application Default Credential Flow, mostly used for testing"],["ApplicationSecret","Represents either ‘installed’ or ‘web’ applications in a json secrets file. See `ConsoleApplicationSecret` for more information"],["AuthorizedUserAuthenticator","Create an authenticator that uses an authorized user credentials."],["ConsoleApplicationSecret","A type to facilitate reading and writing the json secret file as returned by the google developer console"],["DeviceFlowAuthenticator","Create an authenticator that uses the device flow."],["InstalledFlowAuthenticator","Create an authenticator that uses the installed flow."],["ServiceAccountAuthenticator","Create an authenticator that uses a service account."],["ServiceAccountImpersonationAuthenticator","Create a access token authenticator that uses user secrets to impersonate a service account."],["ServiceAccountKey","JSON schema of secret service account key."]]};