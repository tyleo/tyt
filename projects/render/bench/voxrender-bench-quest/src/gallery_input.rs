use crate::{Error, Result, XrSystem};
use openxr::{
    Action, ActionSet, ActiveActionSet, Binding, Path, Session, Vector2f, Vulkan as XrVulkan,
};
use voxrender_bench::GalleryControls;

/// The interaction profile the bindings below belong to. The runtime maps
/// it onto the controllers a headset has.
const PROFILE: &str = "/interaction_profiles/oculus/touch_controller";

/// The Touch controllers as a gallery's controls:
///
/// 1. A or X steps to the next model, and B or Y to the one before
/// 2. The left stick walks, and a press of either stick switches the model
///    between the table and life size
/// 3. The right stick turns with a lean to a side and rises or sinks with
///    a push ahead or back
/// 4. The right trigger grows the model, and the left one shrinks it
pub struct GalleryInput {
    set: ActionSet,

    next: Action<bool>,

    previous: Action<bool>,

    resize: Action<bool>,

    walk: Action<Vector2f>,

    steer: Action<Vector2f>,

    grow: Action<f32>,

    shrink: Action<f32>,
}

impl GalleryInput {
    /// Creates the actions and attaches them to `session`. The session takes
    /// no other action set after.
    pub fn new(xr: &XrSystem, session: &Session<XrVulkan>) -> Result<Self> {
        let instance = &xr.instance;

        let set = instance
            .create_action_set("gallery", "Model gallery", 0)
            .map_err(Error::Xr)?;

        let next = set
            .create_action("next", "Next model", &[])
            .map_err(Error::Xr)?;
        let previous = set
            .create_action("previous", "Model before", &[])
            .map_err(Error::Xr)?;
        let resize = set
            .create_action("resize", "Table or life size", &[])
            .map_err(Error::Xr)?;
        let walk = set.create_action("walk", "Walk", &[]).map_err(Error::Xr)?;
        let steer = set
            .create_action("steer", "Turn and rise", &[])
            .map_err(Error::Xr)?;
        let grow = set
            .create_action("grow", "Grow the model", &[])
            .map_err(Error::Xr)?;
        let shrink = set
            .create_action("shrink", "Shrink the model", &[])
            .map_err(Error::Xr)?;

        let path = |text: &str| instance.string_to_path(text).map_err(Error::Xr);

        instance
            .suggest_interaction_profile_bindings(
                path(PROFILE)?,
                &[
                    Binding::new(&next, path("/user/hand/right/input/a/click")?),
                    Binding::new(&next, path("/user/hand/left/input/x/click")?),
                    Binding::new(&previous, path("/user/hand/right/input/b/click")?),
                    Binding::new(&previous, path("/user/hand/left/input/y/click")?),
                    Binding::new(&resize, path("/user/hand/left/input/thumbstick/click")?),
                    Binding::new(&resize, path("/user/hand/right/input/thumbstick/click")?),
                    Binding::new(&walk, path("/user/hand/left/input/thumbstick")?),
                    Binding::new(&steer, path("/user/hand/right/input/thumbstick")?),
                    Binding::new(&grow, path("/user/hand/right/input/trigger/value")?),
                    Binding::new(&shrink, path("/user/hand/left/input/trigger/value")?),
                ],
            )
            .map_err(Error::Xr)?;

        session.attach_action_sets(&[&set]).map_err(Error::Xr)?;

        Ok(GalleryInput {
            set,
            next,
            previous,
            resize,
            walk,
            steer,
            grow,
            shrink,
        })
    }

    /// Reads the controllers as they stand now. A button counts on the
    /// frame its press starts. Every control rests while the session lacks
    /// focus.
    pub fn read(&self, session: &Session<XrVulkan>) -> Result<GalleryControls> {
        session
            .sync_actions(&[ActiveActionSet::new(&self.set)])
            .map_err(Error::Xr)?;

        let pressed = |action: &Action<bool>| {
            action
                .state(session, Path::NULL)
                .map(|state| state.changed_since_last_sync && state.current_state)
                .map_err(Error::Xr)
        };

        let stick = |action: &Action<Vector2f>| {
            action
                .state(session, Path::NULL)
                .map(|state| state.current_state)
                .map_err(Error::Xr)
        };

        let pulled = |action: &Action<f32>| {
            action
                .state(session, Path::NULL)
                .map(|state| state.current_state)
                .map_err(Error::Xr)
        };

        let walk = stick(&self.walk)?;
        let steer = stick(&self.steer)?;

        Ok(GalleryControls {
            step: i32::from(pressed(&self.next)?) - i32::from(pressed(&self.previous)?),
            resize: pressed(&self.resize)?,
            walk: [walk.x, walk.y],
            turn: steer.x,
            rise: steer.y,
            grow: pulled(&self.grow)? - pulled(&self.shrink)?,
        })
    }
}
