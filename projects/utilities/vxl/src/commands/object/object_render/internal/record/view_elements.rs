use crate::{
    CliValue, Error, Result,
    commands::{PoseTransformEntry, ProjectionKind, TransformFrame, ViewEntry},
};
use ty_math::TyVector3F64;
use voxsmith::operations::object::{
    FitOrFixed, PoseTransform, Rotation, ViewProjection, ViewRecord,
};

/// A view's elements as the flags set them. A flag setting an element twice
/// errors, and the profile stack's entry fills whatever no flag set.
#[derive(Debug)]
pub struct ViewElements {
    name: String,

    frame: Option<TransformFrame>,

    node: Option<String>,

    position: Option<TyVector3F64>,

    rotation: Option<Rotation>,

    orbit: Option<PoseTransform>,

    projection: Option<ProjectionKind>,

    fov: Option<f64>,

    scale: Option<FitOrFixed>,

    select: Vec<String>,
}

impl ViewElements {
    /// The empty elements of the view `name`.
    pub(crate) fn new(name: String) -> Self {
        ViewElements {
            name,
            frame: None,
            node: None,
            position: None,
            rotation: None,
            orbit: None,
            projection: None,
            fov: None,
            scale: None,
            select: Vec::new(),
        }
    }

    /// Sets the frame the position and rotation are read in, which `flag`
    /// gives.
    pub(crate) fn set_frame(&mut self, flag: &str, frame: TransformFrame) -> Result<()> {
        self.check_posed(flag)?;

        claim(&mut self.frame, frame, flag, &self.name, "frame")
    }

    /// Sets the node path the `node` frame reads, which `flag` gives.
    pub(crate) fn set_node(&mut self, flag: &str, path: String) -> Result<()> {
        self.check_posed(flag)?;

        claim(&mut self.node, path, flag, &self.name, "node path")
    }

    /// Sets the position, which `flag` gives.
    pub(crate) fn set_position(&mut self, flag: &str, position: TyVector3F64) -> Result<()> {
        self.check_posed(flag)?;

        claim(&mut self.position, position, flag, &self.name, "position")
    }

    /// Sets the rotation, which `flag` gives.
    pub(crate) fn set_rotation(&mut self, flag: &str, rotation: Rotation) -> Result<()> {
        self.check_posed(flag)?;

        claim(&mut self.rotation, rotation, flag, &self.name, "rotation")
    }

    /// Sets the whole transform to the orbit `flag` gives.
    pub(crate) fn set_orbit(&mut self, flag: &str, orbit: PoseTransform) -> Result<()> {
        if self.is_posed() {
            return Err(Error::usage(format!(
                "{flag} sets view `{}`'s transform, which --view-frame, --view-node, \
                 --view-position, or a rotation flag sets already",
                self.name
            )));
        }

        claim(&mut self.orbit, orbit, flag, &self.name, "transform")
    }

    /// Sets the projection kind, which `flag` gives.
    pub(crate) fn set_projection(&mut self, flag: &str, projection: ProjectionKind) -> Result<()> {
        claim(
            &mut self.projection,
            projection,
            flag,
            &self.name,
            "projection",
        )
    }

    /// Sets the vertical field of view in degrees, which `flag` gives.
    pub(crate) fn set_fov(&mut self, flag: &str, fov: f64) -> Result<()> {
        claim(&mut self.fov, fov, flag, &self.name, "fov")
    }

    /// Sets the orthographic scale, which `flag` gives.
    pub(crate) fn set_scale(&mut self, flag: &str, scale: FitOrFixed) -> Result<()> {
        claim(&mut self.scale, scale, flag, &self.name, "scale")
    }

    /// Adds a hierarchy-path glob to the subject selectors.
    pub(crate) fn push_select(&mut self, glob: String) {
        self.select.push(glob);
    }

    /// The view record the elements lower into over `entry`, the profile
    /// stack's view of the same name. A flag's element stands, and the entry
    /// fills the rest. Errors if the view has no transform, a posed transform
    /// lacks a part, the frame is `camera`, a node path sits under another
    /// frame, or a length is set under the projection that ignores it.
    pub(crate) fn finish(self, entry: Option<&ViewEntry>) -> Result<ViewRecord> {
        let posed = self.is_posed();
        let name = self.name;

        let transform = if let Some(orbit) = self.orbit {
            orbit
        } else if !posed {
            entry
                .and_then(|entry| entry.transform.clone())
                .map(PoseTransformEntry::into_transform)
                .ok_or_else(|| {
                    Error::usage(format!(
                        "view `{name}` has no transform; give it --view-orbit, or --view-frame \
                         with --view-position and a rotation flag"
                    ))
                })?
        } else {
            let missing: Vec<&str> = [
                (self.frame.is_none(), "--view-frame"),
                (self.position.is_none(), "--view-position"),
                (self.rotation.is_none(), "a rotation flag"),
            ]
            .into_iter()
            .filter_map(|(missing, flag)| missing.then_some(flag))
            .collect();

            if !missing.is_empty() {
                return Err(Error::usage(format!(
                    "view `{name}`'s transform lacks {}; a posed view takes --view-frame, \
                     --view-position, and a rotation flag",
                    missing.join(" and ")
                )));
            }

            let frame = self.frame.expect("the check above found a frame");
            let position = self.position.expect("the check above found a position");
            let rotation = self.rotation.expect("the check above found a rotation");

            if frame != TransformFrame::Node
                && let Some(path) = self.node
            {
                return Err(Error::usage(format!(
                    "view `{name}` reads the node path `{path}`, which applies under the node \
                     frame, and its frame is {}",
                    frame.name()
                )));
            }

            match frame {
                TransformFrame::World => PoseTransform::World { position, rotation },

                TransformFrame::Subject => PoseTransform::Subject { position, rotation },

                TransformFrame::Camera => {
                    return Err(Error::usage(format!(
                        "view `{name}`'s frame is camera, and a view's frame is world, subject, \
                         or node"
                    )));
                }

                TransformFrame::Node => PoseTransform::Node {
                    path: self.node.ok_or_else(|| {
                        Error::usage(format!(
                            "view `{name}`'s frame is node, and its transform lacks --view-node"
                        ))
                    })?,
                    position,
                    rotation,
                },
            }
        };

        let projection = self
            .projection
            .or(entry
                .and_then(|entry| entry.projection)
                .map(|projection| projection.0))
            .unwrap_or(ProjectionKind::Perspective);

        let fov = self
            .fov
            .or(entry.and_then(|entry| entry.fov).map(|fov| fov.0));

        let scale = self.scale.or(entry
            .and_then(|entry| entry.scale)
            .map(|scale| FitOrFixed::Fixed(scale.0)));

        let projection = match projection {
            ProjectionKind::Perspective => {
                if scale.is_some() {
                    return Err(Error::usage(format!(
                        "view `{name}` sets a scale, which applies under orthographic, and its \
                         projection is perspective"
                    )));
                }

                let fov = fov.unwrap_or(35.0);

                if fov >= 180.0 {
                    return Err(Error::usage(format!(
                        "view `{name}`'s fov is {fov}, and a field of view is less than 180 \
                         degrees"
                    )));
                }

                ViewProjection::Perspective { fov }
            }

            ProjectionKind::Orthographic => {
                if fov.is_some() {
                    return Err(Error::usage(format!(
                        "view `{name}` sets a fov, which applies under perspective, and its \
                         projection is orthographic"
                    )));
                }

                ViewProjection::Orthographic {
                    scale: scale.unwrap_or(FitOrFixed::Fit),
                }
            }
        };

        let select = if self.select.is_empty() {
            entry
                .and_then(|entry| entry.select.clone())
                .unwrap_or_default()
        } else {
            self.select
        };

        Ok(ViewRecord {
            name,
            transform,
            projection,
            select,
        })
    }

    /// Whether a flag posed the view.
    fn is_posed(&self) -> bool {
        self.frame.is_some()
            || self.node.is_some()
            || self.position.is_some()
            || self.rotation.is_some()
    }

    /// Errors when `flag` poses a view whose transform an orbit set already.
    fn check_posed(&self, flag: &str) -> Result<()> {
        if self.orbit.is_some() {
            return Err(Error::usage(format!(
                "{flag} sets view `{}`'s transform, which --view-orbit sets already",
                self.name
            )));
        }

        Ok(())
    }
}

/// Fills `slot` with `value`, the `element` of the view `name` that `flag`
/// gives. A slot filled already errors.
fn claim<T>(slot: &mut Option<T>, value: T, flag: &str, name: &str, element: &str) -> Result<()> {
    if slot.is_some() {
        return Err(Error::usage(format!(
            "{flag} sets view `{name}`'s {element}, which is set already"
        )));
    }

    *slot = Some(value);

    Ok(())
}
