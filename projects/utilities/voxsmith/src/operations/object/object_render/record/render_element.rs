use branded_id::U32Id;
use std::fmt::{Display, Formatter, Result as FmtResult};
use voxrender::BRenderLight;

/// The element of a [`RenderRecord`](crate::operations::object::RenderRecord)
/// an error rose from.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RenderElement {
    /// The bloom values.
    Bloom,

    /// The image size.
    Image,

    /// A light's transform.
    LightTransform {
        /// The light.
        light_id: U32Id<BRenderLight>,
    },

    /// A view's projection.
    ViewProjection {
        /// The view's name.
        name: String,
    },

    /// A view's subject selectors.
    ViewSelect {
        /// The view's name.
        name: String,
    },

    /// A view's transform.
    ViewTransform {
        /// The view's name.
        name: String,
    },

    /// The view table.
    Views,

    /// The voxel size.
    VoxelSize,
}

impl Display for RenderElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RenderElement::Bloom => f.write_str("the bloom"),

            RenderElement::Image => f.write_str("the image"),

            RenderElement::LightTransform { light_id } => {
                write!(f, "light {}'s transform", light_id.to_u32())
            }

            RenderElement::ViewProjection { name } => write!(f, "view {name}'s projection"),

            RenderElement::ViewSelect { name } => write!(f, "view {name}'s select"),

            RenderElement::ViewTransform { name } => write!(f, "view {name}'s transform"),

            RenderElement::Views => f.write_str("the view table"),

            RenderElement::VoxelSize => f.write_str("the voxel size"),
        }
    }
}
