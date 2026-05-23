use crate::AppError;

pub fn parse_config(config: &str) -> Result<String, AppError> {
    let parsed: toml::Value = toml::from_str(config).map_err(AppError::ParseConfigToml)?;
    let toolchain = parsed
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or(AppError::MissingConfigField("toolchain.name"))?;
    Ok(toolchain.to_string())
}
