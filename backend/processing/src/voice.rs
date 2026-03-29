use aws_sdk_transcribe::types::MediaFormat;
use regex::Regex;
use uuid::Uuid;

use crate::Ctx;
use crate::llm::{build_text_prompt, invoke_claude, parse_json_from_text};

/// Map a MIME type string to a Transcribe MediaFormat.
fn map_media_format(mime_type: &str) -> MediaFormat {
    if mime_type.contains("mp3") {
        MediaFormat::Mp3
    } else if mime_type.contains("mp4") {
        MediaFormat::Mp4
    } else if mime_type.contains("wav") {
        MediaFormat::Wav
    } else if mime_type.contains("flac") {
        MediaFormat::Flac
    } else if mime_type.contains("ogg") {
        MediaFormat::Ogg
    } else {
        MediaFormat::Webm
    }
}

/// Start an AWS Transcribe job and poll until complete. Returns the transcript text.
pub async fn transcribe_voice(
    ctx: &Ctx,
    voice_key: &str,
    mime_type: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let s3_uri = format!("s3://{}/{}", ctx.media_bucket, voice_key);
    let job_name = format!(
        "tastebase-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    tracing::info!(job_name = %job_name, s3_uri = %s3_uri, "starting transcription");

    ctx.transcribe
        .start_transcription_job()
        .transcription_job_name(&job_name)
        .language_code(
            std::env::var("TRANSCRIBE_LANGUAGE")
                .unwrap_or_else(|_| "en-US".into())
                .as_str()
                .into(),
        )
        .media(
            aws_sdk_transcribe::types::Media::builder()
                .media_file_uri(&s3_uri)
                .build(),
        )
        .media_format(map_media_format(mime_type))
        .send()
        .await?;

    poll_transcription_job(ctx, &job_name).await
}

/// Poll a Transcribe job until it completes, fails, or times out.
async fn poll_transcription_job(
    ctx: &Ctx,
    job_name: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let poll_ms: u64 = std::env::var("TRANSCRIBE_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1500);
    let max_polls: u32 = std::env::var("TRANSCRIBE_MAX_POLLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40);

    for _ in 0..max_polls {
        tokio::time::sleep(std::time::Duration::from_millis(poll_ms)).await;

        let resp = ctx
            .transcribe
            .get_transcription_job()
            .transcription_job_name(job_name)
            .send()
            .await?;

        let job = resp
            .transcription_job()
            .ok_or("missing transcription job in response")?;

        match job.transcription_job_status() {
            Some(status) if status.as_str() == "COMPLETED" => {
                let transcript_uri = job
                    .transcript()
                    .and_then(|t| t.transcript_file_uri())
                    .ok_or("transcription completed without transcript URI")?;

                let transcript = fetch_transcript_text(transcript_uri).await?;
                tracing::info!(job_name = %job_name, "transcription completed");
                return Ok(transcript);
            }
            Some(status) if status.as_str() == "FAILED" => {
                let reason = job.failure_reason().unwrap_or("unknown");
                return Err(format!("transcription job failed: {reason}").into());
            }
            _ => continue,
        }
    }

    Err("transcription job timed out".into())
}

/// Fetch the transcript text from the Transcribe result URI.
async fn fetch_transcript_text(
    uri: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let resp = reqwest::get(uri).await?;
    let json: serde_json::Value = resp.json().await?;
    let transcript = json
        .pointer("/results/transcripts/0/transcript")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Ok(transcript.to_string())
}

// -- Notes formatting --

/// Known label normalization map.
fn known_label(label: &str) -> Option<&'static str> {
    match label.trim().to_lowercase().as_str() {
        "flavor" | "flavour" => Some("Flavor"),
        "aroma" => Some("Aroma"),
        "texture" => Some("Texture"),
        "heat" | "heat level" | "spice" | "spice level" => Some("Heat"),
        "pairings" | "pairing" => Some("Pairings"),
        "finish" => Some("Finish"),
        "description" => Some("Description"),
        _ => None,
    }
}

/// Normalize a label to title case, using known labels when possible.
fn format_notes_label(label: &str) -> String {
    let cleaned = label.trim();
    if cleaned.is_empty() {
        return "Note".to_string();
    }
    if let Some(known) = known_label(cleaned) {
        return known.to_string();
    }
    // Title-case the label
    cleaned
        .replace(['_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format lines as bullet points.
fn format_bullet_lines(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            if line.starts_with('-') || line.starts_with('\u{2022}') {
                line.to_string()
            } else {
                format!("- {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert the LLM notes output to a formatted string.
fn notes_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            let lines: Vec<&str> = s.split('\n').collect();
            format_bullet_lines(&lines)
        }
        serde_json::Value::Array(arr) => {
            let lines: Vec<&str> = arr
                .iter()
                .filter_map(|item| item.as_str())
                .filter(|s| !s.trim().is_empty())
                .collect();
            format_bullet_lines(&lines)
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<String> = obj
                .iter()
                .filter_map(|(key, val)| {
                    let s = val.as_str()?;
                    if s.trim().is_empty() {
                        return None;
                    }
                    Some(format!("{}: {}", format_notes_label(key), s.trim()))
                })
                .collect();
            if entries.is_empty() {
                return String::new();
            }
            let lines: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
            format_bullet_lines(&lines)
        }
        _ => String::new(),
    }
}

/// Strip numeric ratings, scores, and filler words from notes.
fn strip_ratings_from_notes(notes: &str) -> String {
    let patterns = [
        r"\b\d+\s*/\s*10\b",
        r"\b\d+\s*/\s*5\b",
        r"\b\d+\s*out of\s*10\b",
        r"(?i)\b(score|rating)\s*[:\-]?\s*\d+(\.\d+)?\b",
        r"(?i)\b(heat|heat level|spice level)\s*[:\-]?\s*\d+(\.\d+)?\b",
    ];

    let mut cleaned = notes.to_string();
    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Remove filler words
    let filler_re = Regex::new(r"(?i)\b(uh|um|erm|er|hmm+)\b").unwrap();
    cleaned = filler_re.replace_all(&cleaned, " ").to_string();

    // Clean up lines
    let lines: Vec<&str> = cleaned
        .split('\n')
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    let result = lines.join("\n");
    // Collapse multiple spaces, clean trailing spaces before periods
    let multi_space = Regex::new(r" {2,}").unwrap();
    let space_dot = Regex::new(r" +\.").unwrap();
    let result = multi_space.replace_all(&result, " ");
    let result = space_dot.replace_all(&result, ".");
    result.trim().to_string()
}

/// Generate a fallback from the raw transcript when LLM formatting fails.
fn fallback_tasting_notes(transcript: &str) -> Option<String> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() || trimmed.len() < 12 {
        return None;
    }
    Some(trimmed.chars().take(1200).collect())
}

/// Run the LLM to format tasting notes from a transcript.
async fn run_notes_llm(
    ctx: &Ctx,
    transcript: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let instructions = "Rewrite this transcript into a concise, professional tasting summary. Identify 2-6 salient categories that are actually mentioned (e.g., Aroma, Flavor, Sweetness, Acidity, Body, Carbonation, Spice/Heat, Balance, Aftertaste). Use category names that fit the product; do not invent or force categories. Return JSON only with key: tasting_notes_user as an object of category -> concise phrase. Keep each phrase under ~12 words. Remove numeric ratings or scores; do not include numbers like 8/10. Remove filler words like um/uh. If the transcript contains no tasting notes, return an empty object. Do not return nested objects or arrays.";

    let payload = build_text_prompt(instructions, transcript);
    let text = invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await?;
    let parsed = match parse_json_from_text(&text) {
        Some(p) => p,
        None => return Ok(String::new()),
    };

    let raw_notes = match parsed.get("tasting_notes_user") {
        Some(v) => notes_to_string(v),
        None => String::new(),
    };

    Ok(raw_notes)
}

pub enum NotesSource {
    Llm,
    Fallback,
    None,
}

pub struct NotesResult {
    pub notes: Option<String>,
    pub source: NotesSource,
}

/// Format voice transcript into structured tasting notes.
/// Tries LLM twice, then falls back to raw transcript.
pub async fn format_tasting_notes(
    ctx: &Ctx,
    transcript: &str,
) -> Result<NotesResult, Box<dyn std::error::Error + Send + Sync>> {
    if transcript.is_empty() {
        return Ok(NotesResult {
            notes: None,
            source: NotesSource::None,
        });
    }

    // First attempt
    let candidate = run_notes_llm(ctx, transcript).await?;
    let cleaned = strip_ratings_from_notes(&candidate);
    if !cleaned.is_empty() {
        return Ok(NotesResult {
            notes: Some(cleaned),
            source: NotesSource::Llm,
        });
    }

    // Second attempt (retry)
    let candidate = run_notes_llm(ctx, transcript).await?;
    let cleaned = strip_ratings_from_notes(&candidate);
    if !cleaned.is_empty() {
        return Ok(NotesResult {
            notes: Some(cleaned),
            source: NotesSource::Llm,
        });
    }

    // Fallback to raw transcript
    let fallback = fallback_tasting_notes(transcript);
    if fallback.is_some() {
        return Ok(NotesResult {
            notes: fallback,
            source: NotesSource::Fallback,
        });
    }

    Ok(NotesResult {
        notes: None,
        source: NotesSource::None,
    })
}

/// Apply formatted notes to the database. Handles force overwrite and
/// needs_attention/attention_reason flags.
pub async fn apply_voice_notes(
    db: &sqlx::PgPool,
    id: Uuid,
    result: &NotesResult,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ref notes) = result.notes {
        if force {
            sqlx::query(
                "UPDATE tastings SET tasting_notes_user = $2, updated_at = now() WHERE id = $1",
            )
            .bind(id)
            .bind(notes)
            .execute(db)
            .await?;
        } else {
            // Only set if current is empty
            sqlx::query(
                "UPDATE tastings SET tasting_notes_user = $2, updated_at = now() WHERE id = $1 AND (tasting_notes_user IS NULL OR tasting_notes_user = '')",
            )
            .bind(id)
            .bind(notes)
            .execute(db)
            .await?;
        }
    }

    // Handle needs_attention flag for fallback notes
    match result.source {
        NotesSource::Fallback => {
            // Check if we should flag attention (force or no existing notes)
            let should_flag = if force {
                true
            } else {
                let row: Option<(String,)> =
                    sqlx::query_as("SELECT tasting_notes_user FROM tastings WHERE id = $1")
                        .bind(id)
                        .fetch_optional(db)
                        .await?;
                match row {
                    Some((notes,)) => notes.trim().is_empty(),
                    None => true,
                }
            };
            if should_flag {
                sqlx::query(
                    "UPDATE tastings SET needs_attention = true, attention_reason = 'Notes fallback used', updated_at = now() WHERE id = $1",
                )
                .bind(id)
                .execute(db)
                .await?;
                tracing::warn!(record_id = %id, "notes fallback used");
            }
        }
        NotesSource::Llm => {
            // Clear fallback attention if previously set
            sqlx::query(
                "UPDATE tastings SET needs_attention = false, attention_reason = NULL, updated_at = now() WHERE id = $1 AND attention_reason = 'Notes fallback used'",
            )
            .bind(id)
            .execute(db)
            .await?;
        }
        NotesSource::None => {}
    }

    Ok(())
}

/// Apply the voice transcript to the database.
pub async fn apply_transcript(
    db: &sqlx::PgPool,
    id: Uuid,
    transcript: &str,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if force {
        sqlx::query("UPDATE tastings SET voice_transcript = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(transcript)
            .execute(db)
            .await?;
    } else {
        sqlx::query(
            "UPDATE tastings SET voice_transcript = $2, updated_at = now() WHERE id = $1 AND (voice_transcript IS NULL OR voice_transcript = '')",
        )
        .bind(id)
        .bind(transcript)
        .execute(db)
        .await?;
    }
    Ok(())
}

/// Format a voice transcript into a recipe review. Reuses the same LLM
/// infrastructure as tasting notes but with a review-specific prompt.
pub async fn format_recipe_review(
    ctx: &Ctx,
    transcript: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let instructions = "You are an editor cleaning up a spoken recipe review. Rules:\n\
         1. Write in first person (I, my, me). Never third person.\n\
         2. Fix all grammar, spelling, and punctuation. Remove double commas, trailing commas, run-on sentences.\n\
         3. Fix misspoken words — if the speaker clearly meant a specific ingredient or cooking term, use the correct spelling (e.g. 'gojutonng' → 'gochujang').\n\
         4. Remove ALL filler words (um, uh, like, you know, sort of, kind of, basically).\n\
         5. Remove the numeric score from the text — it will be extracted separately.\n\
         6. Organize into short paragraphs. Use **bold** for key takeaways.\n\
         7. Keep the speaker's personality and opinions. Don't add information they didn't say.\n\
         8. Return ONLY the cleaned review text. No preamble, no labels, no 'Here is the review'.";

    let payload = build_text_prompt(instructions, transcript);
    match invoke_claude(&ctx.bedrock, &ctx.bedrock_model_id, &payload).await {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                Ok(strip_filler(transcript))
            } else {
                Ok(trimmed)
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "LLM review formatting failed, using raw transcript");
            Ok(strip_filler(transcript))
        }
    }
}

fn strip_filler(text: &str) -> String {
    let re = Regex::new(r"(?i)\b(um|uh|like,?\s)").unwrap();
    re.replace_all(text, "").trim().to_string()
}
