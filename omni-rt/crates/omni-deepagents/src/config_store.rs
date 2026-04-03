use omni_zenfs as zenfs;

const CONFIG_DIR: &str = "/home/config";
const ENV_FILE: &str = "/home/config/.env";
const DEFAULT_MODEL_FILE: &str = "/home/config/default_model";

async fn read_env() -> Result<String, std::io::Error> {
    if !zenfs::exists(ENV_FILE).await? {
        return Ok(String::new());
    }
    let data = zenfs::read_file(ENV_FILE).await?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

async fn write_env(content: &str) -> Result<(), std::io::Error> {
    zenfs::mkdir(CONFIG_DIR, true).await?;
    zenfs::write_file(ENV_FILE, content.as_bytes()).await
}

fn env_key(provider: &str) -> String {
    format!("{}_API_KEY", provider.to_uppercase())
}

fn find_key(env: &str, key: &str) -> Option<String> {
    for line in env.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn set_key(env: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = env.lines().map(|l| l.to_string()).collect();
    let new_line = format!("{}={}", key, value);
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(&format!("{}=", key)) {
            *line = new_line.clone();
            found = true;
            break;
        }
    }
    if !found {
        lines.push(new_line);
    }
    lines.join("\n")
}

fn delete_key(env: &str, key: &str) -> String {
    env.lines()
        .filter(|l| !l.starts_with(&format!("{}=", key)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn get_api_key(provider: &str) -> Result<Option<String>, std::io::Error> {
    let env = read_env().await?;
    Ok(find_key(&env, &env_key(provider)))
}

pub async fn has_api_key(provider: &str) -> Result<bool, std::io::Error> {
    Ok(get_api_key(provider).await?.is_some())
}

pub async fn set_api_key(provider: &str, key_value: &str) -> Result<(), std::io::Error> {
    let env = read_env().await?;
    let key = env_key(provider);
    write_env(&set_key(&env, &key, key_value)).await
}

pub async fn delete_api_key(provider: &str) -> Result<(), std::io::Error> {
    let env = read_env().await?;
    let key = env_key(provider);
    write_env(&delete_key(&env, &key)).await
}

pub async fn get_default_model() -> Result<String, std::io::Error> {
    if !zenfs::exists(DEFAULT_MODEL_FILE).await? {
        return Ok("claude-3-7-sonnet".to_string());
    }
    let data = zenfs::read_file(DEFAULT_MODEL_FILE).await?;
    Ok(String::from_utf8_lossy(&data).trim().to_string())
}

pub async fn get_stored_default_model() -> Result<Option<String>, std::io::Error> {
    if !zenfs::exists(DEFAULT_MODEL_FILE).await? {
        return Ok(None);
    }
    let data = zenfs::read_file(DEFAULT_MODEL_FILE).await?;
    let model_id = String::from_utf8_lossy(&data).trim().to_string();
    if model_id.is_empty() {
        return Ok(None);
    }
    Ok(Some(model_id))
}

pub async fn set_default_model(model_id: &str) -> Result<(), std::io::Error> {
    zenfs::mkdir(CONFIG_DIR, true).await?;
    zenfs::write_file(DEFAULT_MODEL_FILE, model_id.as_bytes()).await
}

pub async fn delete_default_model() -> Result<(), std::io::Error> {
    if zenfs::exists(DEFAULT_MODEL_FILE).await? {
        zenfs::remove(DEFAULT_MODEL_FILE, false).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{delete_key, find_key, set_key};

    #[test]
    fn finds_existing_key() {
        let env = "OPENAI_API_KEY=abc\nANTHROPIC_API_KEY=xyz";
        assert_eq!(find_key(env, "OPENAI_API_KEY"), Some("abc".to_string()));
        assert_eq!(find_key(env, "MISSING"), None);
    }

    #[test]
    fn sets_and_updates_key() {
        let env = "OPENAI_API_KEY=abc";
        let updated = set_key(env, "OPENAI_API_KEY", "def");
        assert_eq!(updated, "OPENAI_API_KEY=def");

        let appended = set_key(env, "ANTHROPIC_API_KEY", "xyz");
        assert!(appended.contains("OPENAI_API_KEY=abc"));
        assert!(appended.contains("ANTHROPIC_API_KEY=xyz"));
    }

    #[test]
    fn deletes_key() {
        let env = "OPENAI_API_KEY=abc\nANTHROPIC_API_KEY=xyz\nOTHER=1";
        let out = delete_key(env, "ANTHROPIC_API_KEY");
        assert!(out.contains("OPENAI_API_KEY=abc"));
        assert!(!out.contains("ANTHROPIC_API_KEY=xyz"));
        assert!(out.contains("OTHER=1"));
    }
}
