use crate::ChannelSource;
use std::str::FromStr;

/// A custom material-map packing: the `<channels>` argument of `--texture-map`,
/// a comma-separated list of `R=<expr>`, `G=<expr>`, `B=<expr>`, and optional
/// `A=<expr>`, each `<expr>` a [`ChannelSource`]. The image's channel count is
/// the highest channel letter present, so naming `A` makes a four-channel image
/// even when `G` and `B` are unnamed; an unnamed channel below the highest is
/// `0`. That is what lets `R=metallic,A=smoothness` reproduce the
/// metallic-smoothness preset.
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelPacking {
    r: Option<ChannelSource>,
    g: Option<ChannelSource>,
    b: Option<ChannelSource>,
    a: Option<ChannelSource>,
}

impl ChannelPacking {
    /// A packing from per-channel sources, each `None` when the channel is not
    /// part of the image; see [`channel_count`](Self::channel_count).
    pub fn new(
        r: Option<ChannelSource>,
        g: Option<ChannelSource>,
        b: Option<ChannelSource>,
        a: Option<ChannelSource>,
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
    /// filled by [`ChannelSource::Zero`].
    pub fn sources(&self) -> Vec<ChannelSource> {
        [&self.r, &self.g, &self.b, &self.a]
            .into_iter()
            .take(self.channel_count())
            .map(|slot| slot.clone().unwrap_or(ChannelSource::Zero))
            .collect()
    }
}

impl FromStr for ChannelPacking {
    type Err = String;

    /// Parses a comma-separated `R=<expr>,G=<expr>,...` channel list. Each
    /// channel must be one of `R`, `G`, `B`, `A`, set at most once, and at least
    /// one channel must be named.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
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
            let source = expr.parse::<ChannelSource>()?;
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
    use crate::{ChannelPacking, ChannelSource};

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            invert,
        }
    }

    #[test]
    fn parses_contiguous_rgb_packing() {
        let packing = "R=metallic,G=smoothness,B=emissive"
            .parse::<ChannelPacking>()
            .unwrap();
        assert_eq!(packing.channel_count(), 3);
        assert_eq!(
            packing.sources(),
            vec![
                attribute("metallic", false),
                attribute("roughness", true),
                attribute("emissive", false),
            ]
        );
    }

    #[test]
    fn alpha_bumps_count_and_zero_fills_gaps() {
        let packing = "R=metallic,A=smoothness".parse::<ChannelPacking>().unwrap();
        assert_eq!(packing.channel_count(), 4);
        assert_eq!(
            packing.sources(),
            vec![
                attribute("metallic", false),
                ChannelSource::Zero,
                ChannelSource::Zero,
                attribute("roughness", true),
            ]
        );
    }

    #[test]
    fn parses_single_channel() {
        let packing = "R=computed-occlusion".parse::<ChannelPacking>().unwrap();
        assert_eq!(packing.channel_count(), 1);
        assert_eq!(packing.sources(), vec![ChannelSource::ComputedOcclusion]);
    }

    #[test]
    fn rejects_bad_packings() {
        assert!("R=metallic,R=roughness".parse::<ChannelPacking>().is_err());
        assert!("X=metallic".parse::<ChannelPacking>().is_err());
        assert!("metallic".parse::<ChannelPacking>().is_err());
        assert!("".parse::<ChannelPacking>().is_err());
    }
}
