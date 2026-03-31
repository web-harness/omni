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

pub async fn get_api_key(provider: &str) -> Result<Option<String>, std::io::Error> {
    let env = read_env().await?;
    let key = env_key(provider);
    for line in env.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Ok(Some(v.trim().to_string()));
            }
        }
    }
    Ok(None)
}

pub async fn has_api_key(provider: &str) -> Result<bool, std::io::Error> {
    Ok(get_api_key(provider).await?.is_some())
}

pub async fn set_api_key(provider: &str, key_value: &str) -> Result<(), std::io::Error> {
    let mut env = read_env().await?;
    let key = env_key(provider);
    let new_line = format!("{}={}", key, key_value);

    let mut lines: Vec<String> = env.lines().map(|l| l.to_string()).collect();
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
    write_env(&lines.join("\n")).await
}

pub async fn delete_api_key(provider: &str) -> Result<(), std::io::Error> {
    let env = read_env().await?;
    let key = env_key(provider);
    let lines: Vec<&str> = env
        .lines()
        .filter(|l| !l.starts_with(&format!("{}=", key)))
        .collect();
    write_env(&lines.join("\n")).await
}

pub async fn get_default_model() -> Result<String, std::io::Error> {
    if !zenfs::exists(DEFAULT_MODEL_FILE).await? {
        return Ok("claude-3-7-sonnet".to_string());
    }
    let data = zenfs::read_file(DEFAULT_MODEL_FILE).await?;
    Ok(String::from_utf8_lossy(&data).trim().to_string())
}

pub async fn set_default_model(model_id: &str) -> Result<(), std::io::Error> {
    zenfs::mkdir(CONFIG_DIR, true).await?;
    zenfs::write_file(DEFAULT_MODEL_FILE, model_id.as_bytes()).await
}
