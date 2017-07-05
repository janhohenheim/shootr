use std::env;

pub fn read_env_var(var: &str) -> String {
    env::var_os(var)
        .expect(&format!(
            "{} must be specified. \
                Did you forget to add it to your .env file?",
            var
        ))
        .into_string()
        .expect(&format!("{} does not contain a valid UTF8 string", var))
}
