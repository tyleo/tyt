use crate::{
    Conv, Dependencies, Error, InputMessage, OaiRequest, Quality, Result, Role, Turn,
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
    /// The next user message in the conversation. If omitted, it is read from
    /// stdin (e.g. `cat prompt.md | tyt oai img`).
    #[arg(value_name = "message")]
    message: Option<String>,

    /// Path to the conversation file to use.
    #[arg(value_name = "conversation-file", long = "conv")]
    conv: Option<PathBuf>,

    /// A system prompt file to prepend, resolved from the configured
    /// `oai.img.systemPromptsDir`. Repeatable; the prompts are prepended in the
    /// order given.
    #[arg(value_name = "system-prompt", long = "system-prompt")]
    system_prompt: Vec<String>,

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

    /// The rendering quality of the generated image.
    #[arg(
        value_name = "quality",
        long,
        value_enum,
        default_value_t = Quality::Auto,
    )]
    quality: Quality,
}

impl Img {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Img {
            message,
            conv,
            system_prompt,
            no_gen,
            continue_kind,
            quality,
        } = self;

        let message = match message {
            Some(message) => message,
            None => dependencies.read_stdin()?,
        };
        let message = message.trim_end().to_owned();
        if message.is_empty() {
            return Err(Error::NoMessage);
        }

        let system_prompts = load_system_prompts(&dependencies, &system_prompt)?;

        let conv_path = conv.unwrap_or_else(|| PathBuf::from("conv.json"));
        let conv_dir = conversation_dir(&conv_path);

        let api_key = dependencies
            .oai_api_key()?
            .ok_or(Error::ApiKeyNotConfigured)?;

        let mut conv = dependencies.read_conv(&conv_path)?.unwrap_or_default();

        let (request, append, system_turns) = build_request(
            &conv,
            &conv_dir,
            continue_kind,
            &message,
            no_gen,
            quality,
            &system_prompts,
        )?;

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
            revised_prompt: None,
            response_id: None,
        };
        let assistant_turn = Turn {
            role: Role::Assistant,
            content: response.text.clone(),
            // The revised prompt only pertains to a generated image, so it is
            // dropped on a text-only (`--no-gen`) turn.
            revised_prompt: image_file.as_ref().and(response.revised_prompt),
            image: image_file.clone(),
            response_id: Some(response.response_id),
        };

        // New conversations open with their `system` turns; continuing in place
        // leaves the prompts already stored at the conversation's start.
        match append {
            Append::InPlace => match conv.conversations.last_mut() {
                Some(conversation) => {
                    conversation.push(user_turn);
                    conversation.push(assistant_turn);
                }
                None => {
                    let mut turns = system_turns;
                    turns.push(user_turn);
                    turns.push(assistant_turn);
                    conv.conversations.push(turns);
                }
            },
            Append::NewConversation => {
                let mut turns = system_turns;
                turns.push(user_turn);
                turns.push(assistant_turn);
                conv.conversations.push(turns);
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

/// Resolves each requested `--system-prompt` file against the configured
/// `oai.img.systemPromptsDir` and reads its contents, preserving order.
fn load_system_prompts(dependencies: &impl Dependencies, names: &[String]) -> Result<Vec<String>> {
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let dir = dependencies
        .system_prompts_dir()?
        .ok_or(Error::SystemPromptsDirNotConfigured)?;

    let mut prompts = Vec::with_capacity(names.len());
    for name in names {
        let path = dir.join(name);
        let content = dependencies
            .read_system_prompt(&path)?
            .ok_or_else(|| Error::SystemPromptNotFound(path.display().to_string()))?;
        prompts.push(content);
    }
    Ok(prompts)
}

/// Builds the API request, decides where its turns are appended, and returns the
/// `system` turns to store at the start of a newly created conversation.
///
/// An explicit `--system-prompt` overrides; otherwise the prior conversation's
/// stored system prompts are inherited so they persist across new conversation
/// threads and `--continue-kind` reconstructions.
fn build_request(
    conv: &Conv,
    conv_dir: &Path,
    continue_kind: ContinueKind,
    message: &str,
    no_gen: bool,
    quality: Quality,
    flag_prompts: &[String],
) -> Result<(OaiRequest, Append, Vec<Turn>)> {
    let generate_image = !no_gen;
    let last = conv.conversations.last();

    let system_prompts: Vec<String> = if flag_prompts.is_empty() {
        last.map(|conversation| stored_system_prompts(conversation))
            .unwrap_or_default()
    } else {
        flag_prompts.to_vec()
    };

    // System prompts are prepended ahead of the request input so they steer the
    // model from the start of the conversation, in the order given.
    let make_request = |previous_response_id, systems: &[String], input: Vec<InputMessage>| {
        let mut full = Vec::with_capacity(systems.len() + input.len());
        full.extend(
            systems
                .iter()
                .map(|prompt| InputMessage::text(Role::System, prompt)),
        );
        full.extend(input);
        OaiRequest {
            model: MODEL.to_owned(),
            previous_response_id,
            input: full,
            generate_image,
            quality,
        }
    };

    let system_turns = |systems: &[String]| {
        systems
            .iter()
            .map(|content| Turn {
                role: Role::System,
                content: content.clone(),
                image: None,
                revised_prompt: None,
                response_id: None,
            })
            .collect()
    };

    match continue_kind {
        ContinueKind::PreviousResponseId => match last.and_then(|c| last_response_id(c)) {
            // Continuing in place; the server retains the system context. Passing
            // --system-prompt here is a mistake, since those prompts are already
            // applied and cannot be changed without rebuilding the conversation.
            Some(previous_response_id) => {
                if !flag_prompts.is_empty() {
                    return Err(Error::SystemPromptOnContinuation);
                }
                Ok((
                    make_request(
                        Some(previous_response_id),
                        &[],
                        vec![InputMessage::user_text(message)],
                    ),
                    Append::InPlace,
                    Vec::new(),
                ))
            }
            None => Ok((
                make_request(
                    None,
                    &system_prompts,
                    vec![InputMessage::user_text(message)],
                ),
                Append::NewConversation,
                system_turns(&system_prompts),
            )),
        },
        ContinueKind::LastImageAllText => Ok((
            make_request(
                None,
                &system_prompts,
                reconstruct_last_image_all_text(last, conv_dir, message),
            ),
            Append::NewConversation,
            system_turns(&system_prompts),
        )),
        ContinueKind::AllImagesAllText => Ok((
            make_request(
                None,
                &system_prompts,
                reconstruct_all_images_all_text(last, conv_dir, message),
            ),
            Append::NewConversation,
            system_turns(&system_prompts),
        )),
        ContinueKind::LastImageOnly => Ok((
            make_request(
                None,
                &system_prompts,
                reconstruct_last_image_only(last, conv_dir, message),
            ),
            Append::NewConversation,
            system_turns(&system_prompts),
        )),
    }
}

/// Returns the contents of the `system` turns stored at the start of a
/// conversation, in order.
fn stored_system_prompts(conversation: &[Turn]) -> Vec<String> {
    conversation
        .iter()
        .filter(|turn| turn.role == Role::System)
        .map(|turn| turn.content.clone())
        .collect()
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

    // System turns are replayed by the request builder, not here, to avoid
    // sending them twice.
    let mut input: Vec<InputMessage> = conversation
        .iter()
        .filter(|turn| turn.role != Role::System)
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
        // System turns are replayed by the request builder, not here.
        if turn.role == Role::System {
            continue;
        }
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
