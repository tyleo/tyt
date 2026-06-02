use crate::{
    Conv, Dependencies, Error, InputMessage, OaiRequest, Result, Role, Turn,
    utilities::ContinueKind,
};
use clap::Parser;
use std::path::{Path, PathBuf};

/// The model used to generate images. Any model supporting the
/// `image_generation` tool works; `gpt-image-1` produces the image itself.
const MODEL: &str = "gpt-5.1";

/// The label prefixing each prior image when reconstructing a conversation so
/// the model recognizes them as its own earlier outputs.
const IMAGE_LABEL: &str = "Your image from the previous conversation:";

/// Generates images from conversations using the OpenAI API.
///
/// Reads or creates a `conv.json` file and, by default, continues the most
/// recent conversation in place via its last `previous_response_id`. If that
/// response is no longer cached by OpenAI, the run fails and asks you to re-run
/// with a `--continue-kind` reconstruction mode that rebuilds context locally.
#[derive(Clone, Debug, Parser)]
#[command(name = "img")]
pub struct Img {
    /// The next user message in the conversation.
    #[arg(value_name = "message")]
    message: String,

    /// Path to the conversation file to use.
    #[arg(value_name = "conversation-file", long = "conv")]
    conv: Option<PathBuf>,

    /// Respond conversationally without generating an image.
    #[arg(value_name = "no-gen", long = "no-gen")]
    no_gen: bool,

    /// How to continue the conversation.
    #[arg(
        value_name = "continue-kind",
        long = "continue-kind",
        value_enum,
        default_value_t = ContinueKind::PreviousResponseId,
    )]
    continue_kind: ContinueKind,
}

impl Img {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Img {
            message,
            conv,
            no_gen,
            continue_kind,
        } = self;

        let conv_path = conv.unwrap_or_else(|| PathBuf::from("conv.json"));
        let conv_dir = conversation_dir(&conv_path);

        let api_key = dependencies
            .oai_api_key()?
            .ok_or(Error::ApiKeyNotConfigured)?;

        let mut conv = dependencies.read_conv(&conv_path)?.unwrap_or_default();

        let (request, append) = build_request(&conv, &conv_dir, continue_kind, &message, no_gen);

        let response = dependencies.generate_image(&api_key, &request)?;

        // Only mutate conv.json after a successful response so a failed run
        // leaves the file untouched and can be retried safely.
        let image_file = match &response.image_png {
            Some(bytes) => {
                let file_name = format!("{}.png", conv.next_image_id);
                dependencies.write_image(&conv_dir.join(&file_name), bytes)?;
                conv.next_image_id += 1;
                Some(file_name)
            }
            None => None,
        };

        let user_turn = Turn {
            role: Role::User,
            content: message,
            image: None,
            response_id: None,
        };
        let assistant_turn = Turn {
            role: Role::Assistant,
            content: response.text.clone(),
            image: image_file.clone(),
            response_id: Some(response.response_id),
        };

        match append {
            Append::InPlace => match conv.conversations.last_mut() {
                Some(conversation) => {
                    conversation.push(user_turn);
                    conversation.push(assistant_turn);
                }
                None => conv.conversations.push(vec![user_turn, assistant_turn]),
            },
            Append::NewConversation => {
                conv.conversations.push(vec![user_turn, assistant_turn]);
            }
        }

        dependencies.write_conv(&conv_path, &conv)?;

        if !response.text.is_empty() {
            dependencies.write_stdout(response.text.as_bytes())?;
            dependencies.write_stdout(b"\n")?;
        }

        if let Some(file_name) = &image_file {
            dependencies.display_image_in_terminal(&conv_dir.join(file_name))?;
            // viuer leaves the cursor directly below the image without a
            // trailing newline, so emit one to end on a fresh line.
            dependencies.write_stdout(b"\n")?;
        }

        Ok(())
    }
}

/// Where the new turns should be appended in `conv.json`.
enum Append {
    /// Continue the most recent conversation in place.
    InPlace,
    /// Start a new conversation.
    NewConversation,
}

/// Returns the directory holding the conversation file and its images.
fn conversation_dir(conv_path: &Path) -> PathBuf {
    match conv_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Builds the API request and decides where its turns are appended, based on the
/// requested continuation mode.
fn build_request(
    conv: &Conv,
    conv_dir: &Path,
    continue_kind: ContinueKind,
    message: &str,
    no_gen: bool,
) -> (OaiRequest, Append) {
    let generate_image = !no_gen;
    let last = conv.conversations.last();

    let new_request = |previous_response_id, input| OaiRequest {
        model: MODEL.to_owned(),
        previous_response_id,
        input,
        generate_image,
    };

    match continue_kind {
        ContinueKind::PreviousResponseId => match last.and_then(|c| last_response_id(c)) {
            Some(previous_response_id) => (
                new_request(
                    Some(previous_response_id),
                    vec![InputMessage::user_text(message)],
                ),
                Append::InPlace,
            ),
            None => (
                new_request(None, vec![InputMessage::user_text(message)]),
                Append::NewConversation,
            ),
        },
        ContinueKind::LastImageAllText => (
            new_request(
                None,
                reconstruct_last_image_all_text(last, conv_dir, message),
            ),
            Append::NewConversation,
        ),
        ContinueKind::AllImagesAllText => (
            new_request(
                None,
                reconstruct_all_images_all_text(last, conv_dir, message),
            ),
            Append::NewConversation,
        ),
        ContinueKind::LastImageOnly => (
            new_request(None, reconstruct_last_image_only(last, conv_dir, message)),
            Append::NewConversation,
        ),
    }
}

/// Returns the most recent assistant `responseId` in a conversation, if any.
fn last_response_id(conversation: &[Turn]) -> Option<String> {
    conversation
        .iter()
        .rev()
        .find_map(|turn| turn.response_id.clone())
}

/// Resolves the absolute path of the final generated image in a conversation.
fn final_image_path(conversation: &[Turn], conv_dir: &Path) -> Option<PathBuf> {
    conversation
        .iter()
        .rev()
        .find_map(|turn| turn.image.as_ref())
        .map(|image| conv_dir.join(image))
}

/// Re-sends every prior text turn plus the final image, attaching the new
/// message to that image in the closing user turn.
fn reconstruct_last_image_all_text(
    last: Option<&Vec<Turn>>,
    conv_dir: &Path,
    message: &str,
) -> Vec<InputMessage> {
    let Some(conversation) = last else {
        return vec![InputMessage::user_text(message)];
    };

    let mut input: Vec<InputMessage> = conversation
        .iter()
        .map(|turn| InputMessage::text(turn.role, &turn.content))
        .collect();

    match final_image_path(conversation, conv_dir) {
        Some(image) => input.push(InputMessage::user_image(
            image,
            Some(message.to_owned()),
            true,
        )),
        None => input.push(InputMessage::user_text(message)),
    }
    input
}

/// Re-sends the full prior history, re-injecting each generated image as a
/// labeled user turn after the assistant turn that produced it.
fn reconstruct_all_images_all_text(
    last: Option<&Vec<Turn>>,
    conv_dir: &Path,
    message: &str,
) -> Vec<InputMessage> {
    let Some(conversation) = last else {
        return vec![InputMessage::user_text(message)];
    };

    let mut input = Vec::new();
    for turn in conversation {
        input.push(InputMessage::text(turn.role, &turn.content));
        if let Some(image) = &turn.image {
            input.push(InputMessage::user_image(
                conv_dir.join(image),
                Some(IMAGE_LABEL.to_owned()),
                false,
            ));
        }
    }
    input.push(InputMessage::user_text(message));
    input
}

/// Sends only the final image and the new message; prior text is dropped.
fn reconstruct_last_image_only(
    last: Option<&Vec<Turn>>,
    conv_dir: &Path,
    message: &str,
) -> Vec<InputMessage> {
    match last.and_then(|conversation| final_image_path(conversation, conv_dir)) {
        Some(image) => vec![InputMessage::user_image(
            image,
            Some(message.to_owned()),
            true,
        )],
        None => vec![InputMessage::user_text(message)],
    }
}
