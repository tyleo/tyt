use crate::{
    Error, MeshOutput, MeshProcessed, MeshRequest, MeshTask, MeshTaskFile, Result, TextureRequest,
    TextureTaskFile,
};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::serde_json::{self, Map, Value, json};

/// The Meshy image-to-3D endpoint, used to create tasks (POST) and retrieve them
/// (GET `/{id}`).
const IMAGE_TO_3D_URL: &str = "https://api.meshy.ai/openapi/v1/image-to-3d";

/// The Meshy retexture endpoint, used to create tasks (POST) and retrieve them
/// (GET `/{id}`).
const RETEXTURE_URL: &str = "https://api.meshy.ai/openapi/v1/retexture";

/// Creates an image-to-3D task and returns its id.
pub(crate) fn create_task(api_key: &str, request: &MeshRequest) -> Result<String> {
    let body = build_create_body(request)?;
    create(IMAGE_TO_3D_URL, api_key, &body)
}

/// Creates a retexture task and returns its id.
pub(crate) fn create_texture_task(api_key: &str, request: &TextureRequest) -> Result<String> {
    let body = build_texture_body(request)?;
    create(RETEXTURE_URL, api_key, &body)
}

/// Posts a create request body to an endpoint and returns the task id.
fn create(url: &str, api_key: &str, body: &Value) -> Result<String> {
    let body_bytes = serde_json::to_vec(body).map_err(|e| Error::Http(e.to_string()))?;

    let authorization = format!("Bearer {api_key}");
    let headers = [
        ("Authorization", authorization.as_str()),
        ("Content-Type", "application/json"),
    ];

    let (status, response_bytes) = tyt_injection::http_post(url, &headers, &body_bytes)
        .map_err(|e| Error::Http(e.to_string()))?;

    parse_create_response(status, &response_bytes)
}

/// Builds the create request body from the stored input, swapping the local
/// `image` (and `texture_image`) paths for the API's base64 `image_url`
/// (`texture_image_url`).
fn build_create_body(request: &MeshRequest) -> Result<Value> {
    let mut body = serde_json::to_value(&request.input).map_err(|e| Error::Http(e.to_string()))?;
    let object = body
        .as_object_mut()
        .ok_or_else(|| Error::Http("input did not serialize to an object".to_owned()))?;

    object.remove("image");
    object.insert(
        "image_url".to_owned(),
        Value::String(image_data_uri(&request.image_path)?),
    );

    if object.remove("texture_image").is_some() {
        let path = request
            .texture_image_path
            .as_ref()
            .ok_or_else(|| Error::Http("missing texture image path".to_owned()))?;
        object.insert(
            "texture_image_url".to_owned(),
            Value::String(image_data_uri(path)?),
        );
    }

    Ok(body)
}

/// Builds the retexture create request body, swapping the local
/// `image_style_url` path for the API's base64 data URI.
fn build_texture_body(request: &TextureRequest) -> Result<Value> {
    let mut body = serde_json::to_value(&request.input).map_err(|e| Error::Http(e.to_string()))?;
    let object = body
        .as_object_mut()
        .ok_or_else(|| Error::Http("input did not serialize to an object".to_owned()))?;

    if object.remove("image_style_url").is_some() {
        let path = request
            .image_style_path
            .as_ref()
            .ok_or_else(|| Error::Http("missing style image path".to_owned()))?;
        object.insert(
            "image_style_url".to_owned(),
            Value::String(image_data_uri(path)?),
        );
    }

    Ok(body)
}

/// Reads the `taskId` from a `*.meshy.mesh.json` (or sibling) task file's bytes.
pub(crate) fn read_task_id(bytes: &[u8]) -> Result<String> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|e| Error::InvalidTaskFile(format!("could not parse task file: {e}")))?;
    value
        .get("taskId")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::InvalidTaskFile("task file is missing 'taskId'".to_owned()))
}

/// Reads an image file and encodes it as a base64 `data:` URI, choosing the MIME
/// type from the file extension.
fn image_data_uri(path: &Path) -> Result<String> {
    let mime = mime_for(path)?;
    let bytes = fs::read(path).map_err(Error::IO)?;
    Ok(format!(
        "data:{mime};base64,{}",
        tyt_injection::encode_base64(&bytes)
    ))
}

/// Returns the MIME type for an image path, erroring on formats Meshy does not
/// accept.
fn mime_for(path: &Path) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg" | "jpeg") => Ok("image/jpeg"),
        other => Err(Error::UnsupportedImageFormat(
            other.unwrap_or("").to_owned(),
        )),
    }
}

/// Parses the create response, returning the task id under the `result` key.
fn parse_create_response(status: u16, bytes: &[u8]) -> Result<String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|e| {
        Error::InvalidResponse(format!("could not parse response (HTTP {status}): {e}"))
    })?;

    if !(200..300).contains(&status) {
        return Err(Error::Api(api_message(&value, status)));
    }

    let result = value
        .get("result")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidResponse("response is missing 'result'".to_owned()))?;
    Ok(result.to_owned())
}

/// Retrieves an image-to-3D task by id.
pub(crate) fn get_task(api_key: &str, task_id: &str) -> Result<MeshTask> {
    get(IMAGE_TO_3D_URL, api_key, task_id)
}

/// Retrieves a retexture task by id.
pub(crate) fn get_texture_task(api_key: &str, task_id: &str) -> Result<MeshTask> {
    get(RETEXTURE_URL, api_key, task_id)
}

/// Retrieves a task by id from the given endpoint.
fn get(base_url: &str, api_key: &str, task_id: &str) -> Result<MeshTask> {
    let url = format!("{base_url}/{task_id}");
    let authorization = format!("Bearer {api_key}");
    let headers = [("Authorization", authorization.as_str())];

    let (status, bytes) =
        tyt_injection::http_get(&url, &headers).map_err(|e| Error::Http(e.to_string()))?;

    parse_task_response(status, bytes)
}

/// Parses the retrieve response into a [`MeshTask`], keeping the verbatim body.
fn parse_task_response(status: u16, bytes: Vec<u8>) -> Result<MeshTask> {
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| {
        Error::InvalidResponse(format!("could not parse task (HTTP {status}): {e}"))
    })?;

    if !(200..300).contains(&status) {
        return Err(Error::Api(api_message(&value, status)));
    }

    let status = value
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidResponse("task is missing 'status'".to_owned()))?
        .to_owned();
    let progress = value.get("progress").and_then(Value::as_u64).unwrap_or(0) as u8;
    let model_urls = string_pairs(value.get("model_urls"));
    let texture_urls = value
        .get("texture_urls")
        .and_then(Value::as_array)
        .and_then(|sets| sets.first())
        .map(texture_pairs)
        .unwrap_or_default();
    let thumbnail_url = value
        .get("thumbnail_url")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let error_message = value
        .get("task_error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .filter(|message| !message.is_empty())
        .map(str::to_owned);

    Ok(MeshTask {
        status,
        progress,
        model_urls,
        texture_urls,
        thumbnail_url,
        error_message,
        raw_json: bytes,
    })
}

/// Collects the string-valued entries of a JSON object as `(key, value)` pairs,
/// preserving their order.
fn string_pairs(value: Option<&Value>) -> Vec<(String, String)> {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|value| (key.clone(), value.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Collects a texture set's map URLs as `(map, url)` pairs in a stable order.
fn texture_pairs(set: &Value) -> Vec<(String, String)> {
    const MAPS: [&str; 5] = ["base_color", "metallic", "normal", "roughness", "emission"];
    let Some(object) = set.as_object() else {
        return Vec::new();
    };
    MAPS.iter()
        .filter_map(|map| {
            object
                .get(*map)
                .and_then(Value::as_str)
                .map(|url| ((*map).to_owned(), url.to_owned()))
        })
        .collect()
}

/// Downloads the bytes at a (pre-signed) URL.
pub(crate) fn download(url: &str) -> Result<Vec<u8>> {
    let (status, bytes) =
        tyt_injection::http_get(url, &[]).map_err(|e| Error::Http(e.to_string()))?;
    if !(200..300).contains(&status) {
        return Err(Error::Api(format!("download failed with HTTP {status}")));
    }
    Ok(bytes)
}

/// Serializes an image-to-3D task file to pretty JSON bytes.
pub(crate) fn mesh_task_file_bytes(file: &MeshTaskFile) -> Result<Vec<u8>> {
    let input = serde_json::to_value(&file.input).map_err(invalid_data)?;
    build_task_file(&file.task_id, "image-to-3d", input, &file.output)
}

/// Serializes a retexture task file to pretty JSON bytes.
pub(crate) fn texture_task_file_bytes(file: &TextureTaskFile) -> Result<Vec<u8>> {
    let input = serde_json::to_value(&file.input).map_err(invalid_data)?;
    build_task_file(&file.task_id, "retexture", input, &file.output)
}

/// Builds a task file's pretty JSON bytes from its id, kind, input, and output.
fn build_task_file(
    task_id: &str,
    task_kind: &str,
    input: Value,
    output: &MeshOutput,
) -> Result<Vec<u8>> {
    let output = match output {
        MeshOutput::Pending => Value::String("pending".to_owned()),
        MeshOutput::Done(done) => {
            let raw: Value = serde_json::from_slice(&done.raw_json).map_err(invalid_data)?;
            json!({ "raw": raw, "processed": processed_value(&done.processed) })
        }
    };

    let root = json!({
        "taskId": task_id,
        "taskKind": task_kind,
        "payload": { "input": input, "output": output },
    });
    serde_json::to_vec_pretty(&root).map_err(invalid_data)
}

/// Builds the `processed` object, omitting empty categories.
fn processed_value(processed: &MeshProcessed) -> Value {
    let mut object = Map::new();
    if !processed.model_files.is_empty() {
        object.insert("modelFiles".to_owned(), pairs_value(&processed.model_files));
    }
    if !processed.texture_files.is_empty() {
        object.insert(
            "textureFiles".to_owned(),
            pairs_value(&processed.texture_files),
        );
    }
    if !processed.thumbnail_files.is_empty() {
        object.insert(
            "thumbnailFiles".to_owned(),
            pairs_value(&processed.thumbnail_files),
        );
    }
    Value::Object(object)
}

/// Builds a JSON object from ordered `(key, value)` string pairs.
fn pairs_value(pairs: &[(String, String)]) -> Value {
    let mut object = Map::new();
    for (key, value) in pairs {
        object.insert(key.clone(), Value::String(value.clone()));
    }
    Value::Object(object)
}

/// Extracts a Meshy error body's `message`, falling back to the HTTP status.
fn api_message(value: &Value, status: u16) -> String {
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unknown error");
    format!("{message} (HTTP {status})")
}

/// Maps a serialization error to an [`Error::IO`] with [`ErrorKind::InvalidData`].
fn invalid_data(e: serde_json::Error) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidData, e))
}
