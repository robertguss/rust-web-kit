//! Google and GitHub OAuth with PKCE. State lives in the session.

use std::str::FromStr;
use std::sync::OnceLock;

use chrono::Utc;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Executor, PgPool, Postgres};
use tower_sessions::Session;
use uuid::Uuid;

use crate::AppError;
use crate::config::Config;
use crate::users::model::User;
use crate::users::{repo, service as users};

const PENDING_KEY: &str = "oauth_pending";

/// Supported providers. Unknown names are 404.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Github => "github",
        }
    }

    fn auth_url(self) -> &'static str {
        match self {
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            Self::Github => "https://github.com/login/oauth/authorize",
        }
    }

    fn token_url(self) -> &'static str {
        match self {
            Self::Google => "https://oauth2.googleapis.com/token",
            Self::Github => "https://github.com/login/oauth/access_token",
        }
    }

    fn scopes(self) -> &'static [&'static str] {
        match self {
            Self::Google => &["openid", "email", "profile"],
            Self::Github => &["read:user", "user:email"],
        }
    }

    fn credentials(self, config: &Config) -> Option<(&str, &str)> {
        match self {
            Self::Google => config.oauth.google(),
            Self::Github => config.oauth.github(),
        }
    }
}

impl FromStr for Provider {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "google" => Ok(Self::Google),
            "github" => Ok(Self::Github),
            _ => Err(()),
        }
    }
}

/// Identity returned by a provider after the code exchange.
#[derive(Debug, Clone)]
pub struct OauthProfile {
    pub provider: Provider,
    pub provider_user_id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingOauth {
    provider: Provider,
    csrf: String,
    pkce_verifier: String,
}

type OauthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// 404 unless this provider has both client id and secret.
pub fn ensure_configured(config: &Config, provider: Provider) -> Result<(), AppError> {
    provider
        .credentials(config)
        .map(|_| ())
        .ok_or(AppError::NotFound)
}

/// Redirect the browser to the provider. Stores PKCE + CSRF in the session.
pub async fn start(
    config: &Config,
    session: &Session,
    provider: Provider,
) -> Result<String, AppError> {
    let client = oauth_client(config, provider)?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(CsrfToken::new_random);
    for scope in provider.scopes() {
        request = request.add_scope(Scope::new((*scope).to_owned()));
    }
    let (url, csrf) = request.set_pkce_challenge(pkce_challenge).url();
    let pending = PendingOauth {
        provider,
        csrf: csrf.secret().clone(),
        pkce_verifier: pkce_verifier.secret().clone(),
    };
    session
        .insert(PENDING_KEY, pending)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(url.to_string())
}

/// Exchange `code`, fetch the profile, link or create the user.
pub async fn finish(
    config: &Config,
    pool: &PgPool,
    session: &Session,
    provider: Provider,
    code: &str,
    state: &str,
) -> Result<User, AppError> {
    let client = oauth_client(config, provider)?;
    let pending = session
        .remove::<PendingOauth>(PENDING_KEY)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    let verifier = take_pending(pending, provider, state)?;
    let token = client
        .exchange_code(AuthorizationCode::new(code.to_owned()))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier))
        .request_async(http_client())
        .await
        .map_err(|error| {
            tracing::error!(error = %error, provider = provider.as_str(), "oauth token exchange failed");
            AppError::BadRequest("oauth token exchange failed".into())
        })?;
    let access_token = token.access_token().secret().clone();
    let profile = fetch_profile(provider, &access_token).await?;
    link_or_create(pool, &profile).await
}

/// Consume session state. Provider and CSRF must match.
fn take_pending(
    pending: Option<PendingOauth>,
    provider: Provider,
    state: &str,
) -> Result<String, AppError> {
    let Some(pending) = pending else {
        return Err(AppError::BadRequest("missing oauth state".into()));
    };
    if pending.provider != provider {
        return Err(AppError::BadRequest("oauth provider mismatch".into()));
    }
    if !secrets_equal(&pending.csrf, state) {
        return Err(AppError::BadRequest("invalid oauth state".into()));
    }
    Ok(pending.pkce_verifier)
}

/// Link by `(provider, provider_user_id)`, else verified email, else create.
pub async fn link_or_create(pool: &PgPool, profile: &OauthProfile) -> Result<User, AppError> {
    if let Some(user_id) =
        find_linked_user_id(pool, profile.provider, &profile.provider_user_id).await?
    {
        return users::find_by_id(pool, user_id)
            .await?
            .ok_or(AppError::NotFound);
    }

    if let Some(existing) = users::find_by_email(pool, &profile.email).await? {
        if existing.email_verified_at.is_none() {
            return Err(AppError::Conflict);
        }
        insert_account(
            pool,
            existing.id,
            profile.provider,
            &profile.provider_user_id,
        )
        .await?;
        return Ok(existing);
    }

    let mut tx = pool.begin().await?;
    let user = repo::create_oauth(&mut *tx, &profile.email, Utc::now()).await?;
    insert_account(
        &mut *tx,
        user.id,
        profile.provider,
        &profile.provider_user_id,
    )
    .await?;
    tx.commit().await?;
    Ok(user)
}

async fn find_linked_user_id<'e, E>(
    executor: E,
    provider: Provider,
    provider_user_id: &str,
) -> Result<Option<Uuid>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar!(
        r#"
        SELECT user_id
        FROM oauth_accounts
        WHERE provider = $1 AND provider_user_id = $2
        "#,
        provider.as_str(),
        provider_user_id,
    )
    .fetch_optional(executor)
    .await
}

async fn insert_account<'e, E>(
    executor: E,
    user_id: Uuid,
    provider: Provider,
    provider_user_id: &str,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query!(
        r#"
        INSERT INTO oauth_accounts (id, user_id, provider, provider_user_id)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        user_id,
        provider.as_str(),
        provider_user_id,
    )
    .execute(executor)
    .await?;
    Ok(())
}

fn oauth_client(config: &Config, provider: Provider) -> Result<OauthClient, AppError> {
    let Some((client_id, client_secret)) = provider.credentials(config) else {
        return Err(AppError::NotFound);
    };
    let redirect = format!(
        "{}/api/auth/oauth/{}/callback",
        config.app_url.trim_end_matches('/'),
        provider.as_str()
    );
    let mut client = BasicClient::new(ClientId::new(client_id.to_owned()))
        .set_client_secret(ClientSecret::new(client_secret.to_owned()))
        .set_auth_uri(
            AuthUrl::new(provider.auth_url().to_owned())
                .map_err(|error| AppError::Internal(error.into()))?,
        )
        .set_token_uri(
            TokenUrl::new(provider.token_url().to_owned())
                .map_err(|error| AppError::Internal(error.into()))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect).map_err(|error| AppError::Internal(error.into()))?,
        );
    if provider == Provider::Github {
        client = client.set_auth_type(AuthType::RequestBody);
    }
    Ok(client)
}

fn http_client() -> &'static oauth2::reqwest::Client {
    static CLIENT: OnceLock<oauth2::reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        oauth2::reqwest::Client::builder()
            .redirect(oauth2::reqwest::redirect::Policy::none())
            .user_agent("rwk")
            .build()
            .expect("oauth http client")
    })
}

async fn fetch_profile(provider: Provider, access_token: &str) -> Result<OauthProfile, AppError> {
    match provider {
        Provider::Google => fetch_google(access_token).await,
        Provider::Github => fetch_github(access_token).await,
    }
}

async fn fetch_google(access_token: &str) -> Result<OauthProfile, AppError> {
    #[derive(Deserialize)]
    struct GoogleUser {
        sub: String,
        email: Option<String>,
        #[serde(default)]
        email_verified: bool,
    }

    let user: GoogleUser = get_json(
        "https://openidconnect.googleapis.com/v1/userinfo",
        access_token,
    )
    .await?;
    if !user.email_verified {
        return Err(AppError::BadRequest(
            "google account email is not verified".into(),
        ));
    }
    let Some(email) = user.email.filter(|email| !email.is_empty()) else {
        return Err(AppError::BadRequest("google account has no email".into()));
    };
    Ok(OauthProfile {
        provider: Provider::Google,
        provider_user_id: user.sub,
        email,
    })
}

async fn fetch_github(access_token: &str) -> Result<OauthProfile, AppError> {
    #[derive(Deserialize)]
    struct GithubUser {
        id: i64,
    }
    #[derive(Deserialize)]
    struct GithubEmail {
        email: String,
        primary: bool,
        verified: bool,
    }

    let user: GithubUser = get_json("https://api.github.com/user", access_token).await?;
    let emails: Vec<GithubEmail> =
        get_json("https://api.github.com/user/emails", access_token).await?;
    let Some(email) = emails
        .into_iter()
        .find(|row| row.primary && row.verified)
        .map(|row| row.email)
    else {
        return Err(AppError::BadRequest(
            "github account has no verified primary email".into(),
        ));
    };
    Ok(OauthProfile {
        provider: Provider::Github,
        provider_user_id: user.id.to_string(),
        email,
    })
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    url: &str,
    access_token: &str,
) -> Result<T, AppError> {
    let response = http_client()
        .get(url)
        .bearer_auth(access_token)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    if !status.is_success() {
        tracing::error!(%status, url, "oauth profile request failed");
        return Err(AppError::BadRequest("oauth profile request failed".into()));
    }
    serde_json::from_str(&body).map_err(|error| AppError::Internal(error.into()))
}

fn secrets_equal(left: &str, right: &str) -> bool {
    Sha256::digest(left.as_bytes()) == Sha256::digest(right.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::password::hash_password;
    use crate::db::MIGRATOR;

    #[test]
    fn take_pending_requires_stored_state() {
        let err = take_pending(None, Provider::Google, "abc").unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn take_pending_rejects_provider_mismatch() {
        let pending = PendingOauth {
            provider: Provider::Google,
            csrf: "csrf-token".into(),
            pkce_verifier: "verifier".into(),
        };
        let err = take_pending(Some(pending), Provider::Github, "csrf-token").unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn take_pending_rejects_wrong_state() {
        let pending = PendingOauth {
            provider: Provider::Google,
            csrf: "csrf-token".into(),
            pkce_verifier: "verifier".into(),
        };
        let err = take_pending(Some(pending), Provider::Google, "nope").unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn take_pending_returns_verifier_on_match() {
        let pending = PendingOauth {
            provider: Provider::Github,
            csrf: "csrf-token".into(),
            pkce_verifier: "pkce-secret".into(),
        };
        let verifier = take_pending(Some(pending), Provider::Github, "csrf-token").unwrap();
        assert_eq!(verifier, "pkce-secret");
    }

    #[test]
    fn unknown_provider_is_none() {
        assert!("gitlab".parse::<Provider>().is_err());
        assert_eq!("google".parse::<Provider>().unwrap(), Provider::Google);
        assert_eq!("github".parse::<Provider>().unwrap(), Provider::Github);
    }

    fn profile(provider: Provider, id: &str, email: &str) -> OauthProfile {
        OauthProfile {
            provider,
            provider_user_id: id.into(),
            email: email.into(),
        }
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn link_creates_verified_user(pool: PgPool) {
        let user = link_or_create(&pool, &profile(Provider::Google, "g-1", "new@example.com"))
            .await
            .unwrap();
        assert_eq!(user.email, "new@example.com");
        assert!(user.password_hash.is_none());
        assert!(user.email_verified_at.is_some());

        let again = link_or_create(&pool, &profile(Provider::Google, "g-1", "new@example.com"))
            .await
            .unwrap();
        assert_eq!(again.id, user.id);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn link_by_verified_email(pool: PgPool) {
        let hash = hash_password("password12").unwrap();
        let existing = repo::create(&pool, "same@example.com", &hash)
            .await
            .unwrap();
        repo::set_verified(&pool, existing.id, Utc::now())
            .await
            .unwrap();

        let linked = link_or_create(
            &pool,
            &profile(Provider::Github, "gh-99", "same@example.com"),
        )
        .await
        .unwrap();
        assert_eq!(linked.id, existing.id);

        let again = link_or_create(
            &pool,
            &profile(Provider::Github, "gh-99", "other@example.com"),
        )
        .await
        .unwrap();
        assert_eq!(again.id, existing.id);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn unverified_email_is_conflict(pool: PgPool) {
        let hash = hash_password("password12").unwrap();
        repo::create(&pool, "open@example.com", &hash)
            .await
            .unwrap();
        let err = link_or_create(&pool, &profile(Provider::Google, "g-2", "open@example.com"))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict));
        let linked: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM oauth_accounts"#)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(linked, 0);
    }
}
