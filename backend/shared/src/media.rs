use aws_sdk_s3::primitives::ByteStream;

use crate::error::AppError;

/// Upload binary data to S3 and return the public URL.
pub async fn upload(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<String, AppError> {
    s3.put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(data))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("S3 upload failed: {e}")))?;

    let url = format!("https://{bucket}.s3.amazonaws.com/{key}");
    Ok(url)
}

/// Download an S3 object as base64-encoded bytes. Returns (base64, content_type).
pub async fn download_base64(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<(String, Option<String>), AppError> {
    use base64::Engine;

    let resp = s3
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("S3 download failed: {e}")))?;

    let content_type = resp.content_type().map(String::from);
    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| AppError::Internal(format!("S3 read failed: {e}")))?
        .into_bytes();

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok((b64, content_type))
}

/// Decode a base64 data URI or raw base64 string into bytes + content type.
pub fn parse_base64_payload(data: &str, fallback_mime: Option<&str>) -> Option<(Vec<u8>, String)> {
    use base64::Engine;

    if data.is_empty() {
        return None;
    }

    let (b64, content_type) = if let Some(rest) = data.strip_prefix("data:") {
        // data:image/jpeg;base64,/9j/...
        let (mime, encoded) = rest.split_once(";base64,")?;
        (encoded, mime.to_string())
    } else {
        (data, fallback_mime.unwrap_or("application/octet-stream").to_string())
    };

    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    Some((bytes, content_type))
}
