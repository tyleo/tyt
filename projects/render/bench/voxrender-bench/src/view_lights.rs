use ty_math::{TyLinSrgbF64, TyVector3F64};
use voxrender::RenderLight;

/// What a view's faces shade under: a scene's first sun and the sum of its
/// hemispheres.
pub struct ViewLights {
    /// The unit direction toward the sun and its light, for a scene with
    /// one.
    pub sun: Option<(TyVector3F64, TyLinSrgbF64)>,

    /// The hemispheres' light from above.
    pub sky: TyLinSrgbF64,

    /// The hemispheres' light from below.
    pub ground: TyLinSrgbF64,
}

impl ViewLights {
    pub fn new(lights: &[RenderLight]) -> Self {
        let black = TyLinSrgbF64::new(0.0, 0.0, 0.0);

        let mut view = ViewLights {
            sun: None,
            sky: black,
            ground: black,
        };

        for light in lights {
            match *light {
                RenderLight::Directional {
                    rotation,
                    color,
                    strength,
                    ..
                } => {
                    if view.sun.is_none() {
                        view.sun = Some((rotation * TyVector3F64::Z, color * strength));
                    }
                }

                RenderLight::Hemisphere {
                    sky,
                    ground,
                    strength,
                } => {
                    view.sky += sky * strength;
                    view.ground += ground * strength;
                }

                _ => {}
            }
        }

        view
    }
}
