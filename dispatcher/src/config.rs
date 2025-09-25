/// Configuration for database connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "storehaus".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            max_connections: 10,
        }
    }
}

impl DatabaseConfig {
    pub fn new(host: String, port: u16, database: String, username: String, password: String) -> Self {
        Self {
            host,
            port,
            database,
            username,
            password,
            max_connections: 10,
        }
    }

    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}