use crate::{
    Conv, Dependencies, Error, InputMessage, OaiRequest, Quality, Result, Role, Turn,
    utilities::ContinueKind,
};
use clap::Parser;
use std::path::{Path, PathBuf};

/// The default model used to generate images, when `--model` is not given. Any
/// model supporting the `image_generation` tool works; `gpt-image-1` produces
/// the image itself.
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

    /// The model used to generate images. Any model supporting the
    /// `image_generation` tool works.
    #[arg(value_name = "model", long, default_value_t = MODEL.to_owned())]
    model: String,

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
            model,
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

        let Plan {
            request,
            append,
            prefix,
            user_image,
        } = build_request(
            &conv,
            &conv_dir,
            continue_kind,
            &message,
            RequestConfig {
                model: &model,
                generate_image: !no_gen,
                quality,
            },
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
            // An image-only reconstruction feeds the prior image back alongside
            // this message, so it is recorded on the user turn.
            image: user_image,
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

        // A new conversation opens with its prefix (system turns plus the prior
        // context the request replayed); continuing in place just appends.
        match append {
            Append::InPlace => match conv.conversations.last_mut() {
                Some(conversation) => {
                    conversation.push(user_turn);
                    conversation.push(assistant_turn);
                }
                None => {
                    let mut turns = prefix;
                    turns.push(user_turn);
                    turns.push(assistant_turn);
                    conv.conversations.push(turns);
                }
            },
            Append::NewConversation => {
                let mut turns = prefix;
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
#[derive(Clone, Copy)]
enum Append {
    /// Continue the most recent conversation in place.
    InPlace,
    /// Start a new conversation.
    NewConversation,
}

/// The model and tool settings shaping the outgoing request.
#[derive(Clone, Copy)]
struct RequestConfig<'a> {
    /// The model to send the request to.
    model: &'a str,
    /// Whether to enable the image-generation tool.
    generate_image: bool,
    /// The rendering quality of the generated image.
    quality: Quality,
}

/// The request to send and how it is recorded in `conv.json`.
struct Plan {
    /// The request to send to OpenAI.
    request: OaiRequest,
    /// Where the new turns are appended.
    append: Append,
    /// Turns stored ahead of the new user turn when a new conversation is
    /// created: the effective system turns followed by the prior context the
    /// request replayed.
    prefix: Vec<Turn>,
    /// The prior image an image-only reconstruction feeds back alongside the new
    /// user message, recorded on the new user turn, if any.
    user_image: Option<String>,
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

/// Builds the API request, decides where its turns are appended, and records the
/// turns to store ahead of the new user turn so a new conversation array mirrors
/// exactly what the request replayed.
///
/// An explicit `--system-prompt` overrides; otherwise the prior conversation's
/// stored system prompts are inherited so they persist across reconstructions —
/// except `new-conversation`, which deliberately starts fresh.
fn build_request(
    conv: &Conv,
    conv_dir: &Path,
    continue_kind: ContinueKind,
    message: &str,
    config: RequestConfig,
    flag_prompts: &[String],
) -> Result<Plan> {
    let last = conv.conversations.last();

    let system_prompts: Vec<String> = if !flag_prompts.is_empty() {
        flag_prompts.to_vec()
    } else if matches!(continue_kind, ContinueKind::NewConversation) {
        // A new conversation deliberately starts fresh, inheriting nothing.
        Vec::new()
    } else {
        last.map(|conversation| stored_system_prompts(conversation))
            .unwrap_or_default()
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
            model: config.model.to_owned(),
            previous_response_id,
            input: full,
            generate_image: config.generate_image,
            quality: config.quality,
        }
    };

    // Continuing in place is the one case that does not rebuild context: the
    // server retains it. Passing --system-prompt here is a mistake, since those
    // prompts are already applied and cannot be changed without rebuilding.
    if let (ContinueKind::PreviousResponseId, Some(previous_response_id)) =
        (continue_kind, last.and_then(|c| last_response_id(c)))
    {
        if !flag_prompts.is_empty() {
            return Err(Error::SystemPromptOnContinuation);
        }
        let request = make_request(
            Some(previous_response_id),
            &[],
            vec![InputMessage::user_text(message)],
        );
        return Ok(Plan {
            request,
            append: Append::InPlace,
            prefix: Vec::new(),
            user_image: None,
        });
    }

    // The last-image modes can only continue from a previously generated image.
    if matches!(
        continue_kind,
        ContinueKind::LastImageAllText | ContinueKind::LastImageOnly
    ) && last
        .and_then(|conversation| final_image_file(conversation))
        .is_none()
    {
        return Err(Error::NoLastImage);
    }

    // Otherwise a new conversation is built from the replayed context. The same
    // turns drive both the request and what is stored, so conv.json records
    // exactly what the model received.
    let (carried, user_image) = context_turns(continue_kind, last);

    let mut input = context_to_input(&carried, conv_dir);
    match &user_image {
        Some(file) => input.push(InputMessage::user_image(
            conv_dir.join(file),
            Some(message.to_owned()),
            true,
        )),
        None => input.push(InputMessage::user_text(message)),
    }
    let request = make_request(None, &system_prompts, input);

    let mut prefix: Vec<Turn> = system_prompts
        .iter()
        .map(|content| Turn {
            role: Role::System,
            content: content.clone(),
            image: None,
            revised_prompt: None,
            response_id: None,
        })
        .collect();
    prefix.extend(carried);

    Ok(Plan {
        request,
        append: Append::NewConversation,
        prefix,
        user_image,
    })
}

/// Returns the prior-conversation turns a reconstruction `--continue-kind`
/// re-sends, plus an image fed back on the new user turn (image-only mode).
///
/// A generated image carried forward as context becomes a "previous
/// conversation" image blob — a `user` turn labelled [`IMAGE_LABEL`] that keeps
/// the image — and existing blobs pass through unchanged, so repeated
/// reconstructions neither duplicate nor relabel them.
fn context_turns(
    continue_kind: ContinueKind,
    last: Option<&Vec<Turn>>,
) -> (Vec<Turn>, Option<String>) {
    let Some(conversation) = last else {
        return (Vec::new(), None);
    };
    let non_system = || conversation.iter().filter(|turn| turn.role != Role::System);

    match continue_kind {
        // No prior context is replayed: previous-response-id leaves it with the
        // server, and new-conversation deliberately starts fresh.
        ContinueKind::PreviousResponseId | ContinueKind::NewConversation => (Vec::new(), None),
        // Every prior turn is re-sent: text as-is, each generated image as a
        // blob after the assistant turn that produced it; existing blobs and
        // other image-bearing user turns carry over unchanged.
        ContinueKind::AllImagesAllText => {
            let mut turns = Vec::new();
            for turn in non_system() {
                match &turn.image {
                    Some(file) if turn.role == Role::Assistant => {
                        turns.push(text_turn(turn));
                        turns.push(image_blob(file));
                    }
                    _ => turns.push(turn.clone()),
                }
            }
            (turns, None)
        }
        // All prior text is re-sent, plus only the final image as a blob. Pure
        // image blobs carry no text and are dropped; other turns keep their
        // text; earlier images are not re-sent.
        ContinueKind::LastImageAllText => {
            let mut turns = Vec::new();
            for turn in non_system() {
                match &turn.image {
                    Some(_) if turn.content == IMAGE_LABEL => {}
                    Some(_) => turns.push(text_turn(turn)),
                    None => turns.push(turn.clone()),
                }
            }
            if let Some(file) = final_image_file(conversation) {
                turns.push(image_blob(&file));
            }
            (turns, None)
        }
        // Only the final image is re-sent, with no prior text to frame it as the
        // model's output, so it is fed back on the new user turn instead.
        ContinueKind::LastImageOnly => (Vec::new(), final_image_file(conversation)),
    }
}

/// Maps stored context turns to request input. A turn carrying an image is sent
/// as a `user` image part with its content as the accompanying text; every other
/// turn is sent as plain text under its own role.
fn context_to_input(turns: &[Turn], conv_dir: &Path) -> Vec<InputMessage> {
    turns
        .iter()
        .map(|turn| match &turn.image {
            Some(file) => {
                InputMessage::user_image(conv_dir.join(file), Some(turn.content.clone()), false)
            }
            None => InputMessage::text(turn.role, &turn.content),
        })
        .collect()
}

/// A copy of `turn` keeping just its authored text, dropping the image and the
/// metadata tied to it.
fn text_turn(turn: &Turn) -> Turn {
    Turn {
        role: turn.role,
        content: turn.content.clone(),
        image: None,
        revised_prompt: None,
        response_id: None,
    }
}

/// A "previous conversation" image blob: a `user` turn labelled [`IMAGE_LABEL`]
/// carrying the image fed back to the model.
fn image_blob(file: &str) -> Turn {
    Turn {
        role: Role::User,
        content: IMAGE_LABEL.to_owned(),
        image: Some(file.to_owned()),
        revised_prompt: None,
        response_id: None,
    }
}

/// Returns the file name of the final generated image in a conversation.
fn final_image_file(conversation: &[Turn]) -> Option<String> {
    conversation
        .iter()
        .rev()
        .find_map(|turn| turn.image.clone())
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
