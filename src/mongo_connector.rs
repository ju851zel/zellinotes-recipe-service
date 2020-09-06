use mongodb::{Client, Database};
use mongodb::error::Error;
use mongodb::options::ClientOptions;

const URL: &str = "mongodb://localhost:27017";
const APP_NAME: &str = "Zellinotes recipes";
const DATABASE: &str = "zellinotes_recipes";

pub async fn start() -> Result<Database, Error> {
    let mut client_options = ClientOptions::parse(URL).await?;
    client_options.app_name = Some(APP_NAME.to_string());
    let client = Client::with_options(client_options)?;
    return Ok(client.database(DATABASE));
}
