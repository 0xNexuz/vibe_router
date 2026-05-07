use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use regex::Regex;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::env;
use std::fs;

// --- 1. OUR STRICT SCHEMA ---
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct AgentTriageResponse {
    error_category: String,
    action_code: String,
    requires_human: bool,
    confidence_score: u8,
    diagnostic_reasoning: String,
}

// --- 2. GEMINI API WRAPPER STRUCTS ---
#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}
#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}
#[derive(Deserialize, Debug)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}
#[derive(Deserialize, Debug)]
struct GeminiPart {
    text: String, 
}

// --- 3. THE FILE SYSTEM SWAPPER (For the Demo) ---
fn execute_key_rotation() {
    println!("🔄 [SYS] Accessing local keystore...");
    sleep(std::time::Duration::from_secs(1)); // Dramatic pause for the demo video
    
    // We write to .env.demo so we don't accidentally nuke your real working key
    let backup_key = "AIzaSy_BACKUP_KEY_INITIALIZED_9942_VIBE";
    let new_env_content = format!("GEMINI_API_KEY={}\n# Autonomously swapped by VibeRouter after 403 error.", backup_key);
    
    fs::write(".env.demo", new_env_content).expect("Failed to write to .env.demo");
    println!("✅ [SYS] Backup key injected into .env.demo.");
    println!("▶️ [SYS] State recovered. Resuming main execution thread...\n");
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let api_key = env::var("GEMINI_API_KEY")
        .expect("🚨 GEMINI_API_KEY not found in environment! Add it to your .env file.");

    let (tx, mut rx1) = broadcast::channel::<String>(100);
    println!("🚀 VibeRouter Initialized. Starting async nodes...\n");

    // --- 4. THE EXECUTION ENGINE (The Muscle) ---
    let tx_engine = tx.clone();
    tokio::spawn(async move {
        let logs = vec![
            "INFO: Booting Rust edge node...",
            "INFO: Connecting to Solana mainnet RPC...",
            "INFO: Fetching user vault data...",
            "ERROR: 403 Forbidden: CONSUMER_SUSPENDED", 
            "FATAL: Execution halted.",
        ];

        for log in logs {
            sleep(Duration::from_secs(1)).await;
            println!("[ENGINE STDOUT] {}", log);
            let _ = tx_engine.send(log.to_string());
        }
    });

    // --- 5. THE LOG STREAMER & TRIAGE (The Brain) ---
    let observer_handle = tokio::spawn(async move {
        let api_error_regex = Regex::new(r"403 Forbidden: CONSUMER_SUSPENDED").unwrap();
        let client = Client::new();
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key);

        while let Ok(msg) = rx1.recv().await {
            if api_error_regex.is_match(&msg) {
                println!("\n🚨 [OBSERVER ALERT] Critical Anomaly Detected!");
                println!("⏸️ [OBSERVER ACTION] Pausing thread & routing to AI Triage...\n");

                // Our Heavily Engineered Master Prompt
                let prompt_text = format!(
                    "You are an autonomous DevOps triage agent. Analyze this terminal error: '{}'. \
                    Classify it into 'INFRASTRUCTURE', 'SYNTAX', or 'STATE_MISSING'. \
                    Determine the action code: 'ROTATE_KEY', 'RETRY_DELAY', or 'HALT_FOR_HUMAN'. \
                    CRITICAL RULE: If the error is a 403 Forbidden, rate limit, quota exhaustion, or 'CONSUMER_SUSPENDED', you MUST select 'ROTATE_KEY' as the system has a vault of backup keys ready. Do not halt for a human for API auth errors. \
                    Output ONLY a valid JSON object matching this schema: \
                    {{\"error_category\": \"STRING\", \"action_code\": \"STRING\", \"requires_human\": BOOLEAN, \"confidence_score\": INTEGER, \"diagnostic_reasoning\": \"STRING\"}}",
                    msg
                );

                let payload = serde_json::json!({
                    "contents": [{"parts": [{"text": prompt_text}]}],
                    "generationConfig": {"response_mime_type": "application/json"}
                });

                let res = client.post(&url).json(&payload).send().await;

                match res {
                    Ok(response) => {
                        let raw_text = response.text().await.unwrap_or_default();
                        
                        // Keeping the debug print off to make the demo video look perfectly clean
                        // println!("🔍 [DEBUG] Raw Response: {}", raw_text);

                        if let Ok(gemini_data) = serde_json::from_str::<GeminiResponse>(&raw_text) {
                            if let Some(candidate) = gemini_data.candidates.first() {
                                if let Some(part) = candidate.content.parts.first() {
                                    if let Ok(decision) = serde_json::from_str::<AgentTriageResponse>(&part.text) {
                                        println!("🧠 [TRIAGE DECISION]");
                                        println!("   ├── Category:   {}", decision.error_category);
                                        println!("   ├── Action:     {}", decision.action_code);
                                        println!("   └── Reasoning:  {}\n", decision.diagnostic_reasoning);
                                        
                                        // --- 6. THE LOGIC GATE ---
                                        match decision.action_code.as_str() {
                                            "ROTATE_KEY" => execute_key_rotation(),
                                            "RETRY_DELAY" => println!("⏳ [SYS] Applying exponential backoff..."),
                                            "HALT_FOR_HUMAN" => println!("🛑 [SYS] Unrecoverable state. Paging developer..."),
                                            _ => println!("❓ [SYS] Unknown action code. Halting."),
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => println!("⚠️ Network error: {}", e),
                }
            }
        }
    });

    let _ = observer_handle.await;
    println!("🏁 VibeRouter operations concluded.");
}