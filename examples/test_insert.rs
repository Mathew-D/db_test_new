// Quick test to see what Turso returns on insert
use std::error::Error;

const TURSO_URL: &str = "https://testing-mathew-d.aws-us-east-2.turso.io";
const TURSO_AUTH_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicnciLCJpYXQiOjE3NjYwMDM3MzQsImlkIjoiYWJmN2VjMmQtNjI4Yy00NjQ1LTk5YWEtYjJlN2JkYmRlZjBiIiwicmlkIjoiMTc5YjVmZjktZTFlNC00YjdjLWIxYWQtMmJhYmMwOTBjNjhiIn0.BVSKprWC8aRNmi8oh6O8zHM7GsdF01d5miK3a95-UsljE6DtLk4U_iqJfHJkKA2CmvaBS706pes6I2RSUsBoCw";

fn main() -> Result<(), Box<dyn Error>> {
    let url = format!("{}/v2/pipeline", TURSO_URL);
    
    let body = serde_json::json!({
        "requests": [{
            "type": "execute",
            "stmt": {
                "sql": "INSERT INTO messages (text) VALUES ('Test Message')"
            }
        }]
    });
    
    let body_str = serde_json::to_string(&body)?;
    println!("Request body:\n{}\n", body_str);
    
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", TURSO_AUTH_TOKEN))
        .set("Content-Type", "application/json")
        .send_string(&body_str);
    
    match response {
        Ok(resp) => {
            let resp_text = resp.into_string()?;
            println!("Response:\n{}", resp_text);
            
            // Pretty print the JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp_text) {
                println!("\nPretty JSON:\n{}", serde_json::to_string_pretty(&json)?);
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let error_body = response.into_string().unwrap_or_else(|_| "Could not read error body".to_string());
            println!("HTTP {} error:\n{}", code, error_body);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    
    Ok(())
}
