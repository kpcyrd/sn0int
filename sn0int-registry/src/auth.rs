use crate::github;
use diesel::pg::PgConnection;
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl, TokenResponse};
use sn0int_registry::errors::*;
use sn0int_registry::models::AuthToken;
use url::Url;
use std::env;


pub struct Authenticator {
    client: BasicClient,
}

impl Authenticator {
    pub fn new(client_id: String, client_secret: String, redirect_url: Url) -> Authenticator {
         let auth_url = AuthUrl::new(
            Url::parse("https://github.com/login/oauth/authorize")
                .expect("Invalid authorization endpoint URL")
        );

        let token_url = TokenUrl::new(
            Url::parse("https://github.com/login/oauth/access_token")
                .expect("Invalid token endpoint URL"),
        );

        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);

        // Set up the config for the Github OAuth2 process.
        let client = BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url, Some(token_url)
        )
        .set_redirect_url(RedirectUrl::new(redirect_url));

        Authenticator {
            client,
        }
    }

    pub fn from_env() -> Result<Authenticator> {
        let client_id = env::var("GITHUB_CLIENT_ID")
            .context("GITHUB_CLIENT_ID is not set")?;
        let client_secret = env::var("GITHUB_CLIENT_SECRET")
            .context("GITHUB_CLIENT_SECRET is not set")?;
        let redirect_url = env::var("OAUTH_REDIRECT_URL")
            .context("OAUTH_REDIRECT_URL is not set")?;
        let redirect_url = redirect_url.parse()?;
        Ok(Authenticator::new(client_id, client_secret, redirect_url))
    }

    pub fn request_auth(&self, session: String) -> (Url, CsrfToken) {
        self.client.authorize_url(|| CsrfToken::new(session))
    }

    pub fn store_code(&self, code: String, state: String, connection: &PgConnection) -> Result<()> {
        let code = AuthorizationCode::new(code);

        // TODO: csrf check

        let response = self.client.exchange_code(code)
            .context("Github authentication failed")?;

        let access_token = response.access_token();
        let access_token = access_token.secret().to_string();

        let user = github::get_username(&access_token)?;

        AuthToken::create(&AuthToken {
            id: state,
            author: user,
            access_token,
        }, connection)?;

        Ok(())
    }
}
