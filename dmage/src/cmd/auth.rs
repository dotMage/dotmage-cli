//! `dmage auth` — authenticate, cache AK in keychain.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use dotmage_client::keychain;
use dotmage_client::token::{self, ServerTokens};
use dotmage_client::types::*;
use dotmage_crypto::envelope;
use dotmage_crypto::kdf;

use super::{CliError, Context};

pub fn run(ctx: &mut Context, server: Option<String>, ttl: Option<String>) -> Result<(), CliError> {
    // Save server URL if provided
    if let Some(url) = &server {
        ctx.config.server_url = Some(url.clone());
        ctx.config
            .save()
            .map_err(|e| CliError::Config(e.to_string()))?;
    }

    let ttl_secs = parse_ttl(ttl.as_deref()).unwrap_or(ctx.config.key_ttl_secs);

    let account_exists = ctx.backend.account_exists()?;

    if !account_exists {
        return bootstrap(ctx, ttl_secs);
    }

    // Existing account: download keys, derive MK, unwrap AK
    let keys = ctx.backend.get_account_keys()?;

    let password = prompt_password("Master password: ")?;

    let salt = B64
        .decode(&keys.salt)
        .map_err(|e| CliError::Crypto(e.to_string()))?;
    let salt: [u8; 16] = salt
        .try_into()
        .map_err(|_| CliError::Crypto("invalid salt".into()))?;

    let params = kdf::ArgonParams {
        memory: keys.argon_params.memory,
        iterations: keys.argon_params.iterations,
        parallelism: keys.argon_params.parallelism,
        version: keys.argon_params.version,
    };

    let mk = kdf::derive_master_key_with_params(password.as_bytes(), &salt, &params)
        .map_err(|e| CliError::Crypto(e.to_string()))?;

    let nonce_ak = B64
        .decode(&keys.nonce_ak)
        .map_err(|e| CliError::Crypto(e.to_string()))?;
    let nonce_ak: [u8; 24] = nonce_ak
        .try_into()
        .map_err(|_| CliError::Crypto("invalid nonce".into()))?;
    let wrapped_ct = B64
        .decode(&keys.wrapped_ak)
        .map_err(|e| CliError::Crypto(e.to_string()))?;

    let wrapped = envelope::WrappedAk {
        nonce: nonce_ak,
        ciphertext: wrapped_ct,
    };

    let ak = envelope::unwrap_ak(&mk, &wrapped)
        .map_err(|_| CliError::Other("invalid password".into()))?;

    // Store AK in keychain
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    keychain::store_ak(&server_hash, &ak, ttl_secs)
        .map_err(|e| CliError::Keychain(e.to_string()))?;

    let days = ttl_secs / 86400;
    ctx.success(&format!("Authenticated. Key cached (expires in {days}d)."));
    Ok(())
}

fn bootstrap(ctx: &mut Context, ttl_secs: u64) -> Result<(), CliError> {
    println!("\n  \x1b[36mNo account found — creating new account\x1b[0m\n");

    let password = prompt_password("New master password: ")?;
    let password_confirm = prompt_password("Confirm master password: ")?;
    if password != password_confirm {
        return Err(CliError::Other("passwords do not match".into()));
    }

    // Ask for bootstrap secret if using server
    let bootstrap_secret = if ctx.config.server_url.is_some() {
        prompt_password("Bootstrap secret: ")?
    } else {
        String::new()
    };

    // Generate crypto material locally
    let salt = kdf::generate_salt();
    let mk = kdf::derive_master_key(password.as_bytes(), &salt)
        .map_err(|e| CliError::Crypto(e.to_string()))?;

    let ak = envelope::generate_account_key();
    let wrapped = envelope::wrap_ak(&mk, &ak).map_err(|e| CliError::Crypto(e.to_string()))?;

    let device_name = hostname();

    let req = AccountInitReq {
        salt: B64.encode(salt),
        argon_params: ArgonParamsDto {
            memory: kdf::ARGON2_MEMORY_KIB,
            iterations: kdf::ARGON2_ITERATIONS,
            parallelism: kdf::ARGON2_PARALLELISM,
            version: kdf::ARGON2_VERSION,
        },
        nonce_ak: B64.encode(wrapped.nonce),
        wrapped_ak: B64.encode(&wrapped.ciphertext),
        device_name: device_name.clone(),
        bootstrap_secret,
        salt_rc: None,
        nonce_rc: None,
        wrapped_ak_rc: None,
    };

    let resp = ctx.backend.account_init(&req)?;

    // Store tokens
    let server_hash = keychain::server_hash(&ctx.config.server_id());
    token::save_tokens(
        &server_hash,
        &ServerTokens {
            device_token: resp.device_token,
            refresh_token: resp.refresh_token,
            device_id: resp.account_id,
        },
    )
    .map_err(|e: token::TokenError| CliError::Other(e.to_string()))?;

    // Store AK in keychain
    keychain::store_ak(&server_hash, &ak, ttl_secs)
        .map_err(|e| CliError::Keychain(e.to_string()))?;

    let days = ttl_secs / 86400;
    ctx.success(&format!(
        "Account created. Key cached (expires in {days}d)."
    ));
    Ok(())
}

fn prompt_password(prompt: &str) -> Result<String, CliError> {
    rpassword::prompt_password(prompt).map_err(CliError::Io)
}

fn parse_ttl(s: Option<&str>) -> Option<u64> {
    let s = s?;
    let s = s.trim();
    if let Some(days) = s.strip_suffix('d') {
        days.parse::<u64>().ok().map(|d| d * 86400)
    } else if let Some(hours) = s.strip_suffix('h') {
        hours.parse::<u64>().ok().map(|h| h * 3600)
    } else {
        s.parse::<u64>().ok()
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            #[cfg(unix)]
            {
                let mut buf = [0u8; 256];
                use std::ffi::CStr;
                unsafe {
                    if libc::gethostname(buf.as_mut_ptr() as *mut _, buf.len()) == 0 {
                        if let Ok(s) = CStr::from_ptr(buf.as_ptr() as *const _).to_str() {
                            return Ok(s.to_string());
                        }
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "unknown".into())
}
