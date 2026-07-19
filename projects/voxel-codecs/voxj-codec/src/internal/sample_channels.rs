/// The channel layout of an object's sample block: one channel per sampled
/// layer, in `layers` order (spec rule 11). Built from the per-layer material
/// counts of
/// [`voxj_palette_material_counts`](crate::voxj_palette_material_counts()),
/// it pairs each channel with the layer it samples and that layer's material
/// count, so the encoders, the decoder, and the geometry check all take
/// channel arity, bit widths, and the channel-to-layer mapping from one
/// place.
pub struct SampleChannels {
    /// The layer each channel samples, in `layers` order.
    layers: Vec<usize>,

    /// The material count M of each channel's layer, aligned to
    /// [`layers`](Self::layers).
    counts: Vec<usize>,
}

impl SampleChannels {
    /// Derives the layout from one material count per layer: a layer is
    /// sampled iff its count is above zero.
    pub fn from_material_counts(material_counts: &[usize]) -> Self {
        let layers: Vec<usize> = (0..material_counts.len())
            .filter(|&layer| material_counts[layer] > 0)
            .collect();
        let counts = layers.iter().map(|&layer| material_counts[layer]).collect();
        Self { layers, counts }
    }

    /// The number of channels: one per sampled layer.
    pub fn channels(&self) -> usize {
        self.layers.len()
    }

    /// The layer channel `channel` samples: the `channel`-th layer whose
    /// palette has materials.
    pub fn layer(&self, channel: usize) -> usize {
        self.layers[channel]
    }

    /// The material count M of each channel's layer, one entry per channel.
    pub fn counts(&self) -> &[usize] {
        &self.counts
    }
}

#[cfg(test)]
mod tests {
    use crate::SampleChannels;

    #[test]
    fn skips_unsampled_layers() {
        let layout = SampleChannels::from_material_counts(&[256, 0, 8]);
        assert_eq!(layout.channels(), 2);
        assert_eq!((layout.layer(0), layout.layer(1)), (0, 2));
        assert_eq!(layout.counts(), &[256, 8]);
    }
}
