use crate::{
    Result,
    commands::{PropertyBinding, parse_material_channel, resolve_material_channel},
};
use std::{result::Result as StdResult, str::FromStr};
use voxsmith::{MaterialBake, MaterialChannel};

/// A material-map channel packing: a [`MaterialChannel`] per named RGBA
/// channel.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelPacking {
    r: Option<MaterialChannel>,
    g: Option<MaterialChannel>,
    b: Option<MaterialChannel>,
    a: Option<MaterialChannel>,
}

impl ChannelPacking {
    /// A packing from per-channel sources, each `None` when the channel is not
    /// part of the image; see [`channel_count`](Self::channel_count).
    pub fn new(
        r: Option<MaterialChannel>,
        g: Option<MaterialChannel>,
        b: Option<MaterialChannel>,
        a: Option<MaterialChannel>,
    ) -> Self {
        ChannelPacking { r, g, b, a }
    }

    /// The image's channel count, the index of the highest channel named, from
    /// `1` for an R-only packing to `4` when `A` is present.
    pub fn channel_count(&self) -> usize {
        if self.a.is_some() {
            4
        } else if self.b.is_some() {
            3
        } else if self.g.is_some() {
            2
        } else {
            1
        }
    }

    /// The source feeding each channel up to [`channel_count`](Self::channel_count),
    /// in `R`, `G`, `B`, `A` order, with an unnamed channel below the highest
    /// filled by [`MaterialChannel::Zero`].
    pub fn sources(&self) -> Vec<MaterialChannel> {
        [&self.r, &self.g, &self.b, &self.a]
            .into_iter()
            .take(self.channel_count())
            .map(|slot| slot.clone().unwrap_or(MaterialChannel::Zero))
            .collect()
    }

    /// Resolves every channel against the `--define-property` bindings into
    /// the packing bake with concrete property keys.
    pub(crate) fn resolve(&self, bindings: &[PropertyBinding]) -> Result<MaterialBake> {
        let channels = self
            .sources()
            .iter()
            .map(|channel| resolve_material_channel(channel, bindings))
            .collect::<Result<Vec<_>>>()?;

        Ok(MaterialBake::Packing(channels))
    }
}

impl FromStr for ChannelPacking {
    type Err = String;

    /// Parses a comma-separated `R=<expr>,G=<expr>,...` channel list. Each
    /// channel must be one of `R`, `G`, `B`, `A`, set at most once, and at least
    /// one channel must be named.
    fn from_str(value: &str) -> StdResult<Self, Self::Err> {
        let mut packing = ChannelPacking {
            r: None,
            g: None,
            b: None,
            a: None,
        };

        for part in value.split(',') {
            let (channel, expr) = part
                .split_once('=')
                .ok_or_else(|| format!("`{part}` is not `R=<expr>`"))?;

            let source = parse_material_channel(expr)?;

            let slot = match channel {
                "R" => &mut packing.r,
                "G" => &mut packing.g,
                "B" => &mut packing.b,
                "A" => &mut packing.a,
                _ => return Err(format!("`{channel}` is not a channel; use R, G, B, or A")),
            };

            if slot.is_some() {
                return Err(format!("channel `{channel}` is set twice"));
            }

            *slot = Some(source);
        }

        if packing.r.is_none() && packing.g.is_none() && packing.b.is_none() && packing.a.is_none()
        {
            return Err("a packing names no channels".to_string());
        }

        Ok(packing)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ChannelPacking;
    use voxsmith::{
        MaterialChannel,
        voxcore::material::{EMISSIVE_STRENGTH, METALLIC, ROUGHNESS},
    };

    fn property(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Property {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    #[test]
    fn parses_contiguous_rgb_packing() {
        let packing = "R=metallic,G=1-roughness,B=emissiveStrength"
            .parse::<ChannelPacking>()
            .unwrap();
        assert_eq!(packing.channel_count(), 3);
        assert_eq!(
            packing.sources(),
            vec![
                property(METALLIC, false),
                property(ROUGHNESS, true),
                property(EMISSIVE_STRENGTH, false),
            ]
        );
    }

    #[test]
    fn alpha_bumps_count_and_zero_fills_gaps() {
        let packing = "R=metallic,A=1-roughness"
            .parse::<ChannelPacking>()
            .unwrap();
        assert_eq!(packing.channel_count(), 4);
        assert_eq!(
            packing.sources(),
            vec![
                property(METALLIC, false),
                MaterialChannel::Zero,
                MaterialChannel::Zero,
                property(ROUGHNESS, true),
            ]
        );
    }

    #[test]
    fn parses_single_channel() {
        let packing = "R=computed-occlusion".parse::<ChannelPacking>().unwrap();
        assert_eq!(packing.channel_count(), 1);
        assert_eq!(packing.sources(), vec![MaterialChannel::ComputedOcclusion]);
    }

    #[test]
    fn rejects_bad_packings() {
        assert!("R=metallic,R=roughness".parse::<ChannelPacking>().is_err());
        assert!("X=metallic".parse::<ChannelPacking>().is_err());
        assert!("metallic".parse::<ChannelPacking>().is_err());
        assert!("".parse::<ChannelPacking>().is_err());
    }
}
