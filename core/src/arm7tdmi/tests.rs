//TODO: implement step tests
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct State {}

    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        name: String,
        initial: State,
        r#final: State,
    }
}
