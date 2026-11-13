use crate::{
    Result,
    operations::sdf_doc::{
        EntryPaths, SdfMaterialProperties, SdfPlace, SdfShapes, check_document, check_shape_boxes,
        collect_places, resolve_materials, sample_voxel_size, text_outlines,
    },
    utilities::GridResolution,
};
use branded_id::IdVec;
use sdfcore::{BSdfMaterial, SdfMain};

/// A model that passed sampling's checks, with the parts sampling reads.
pub struct CheckedModel {
    /// Every place of every part, each before its children, from the first
    /// root part.
    pub places: Vec<SdfPlace>,

    /// The properties of each material of the model.
    pub materials: IdVec<BSdfMaterial, SdfMaterialProperties>,

    /// The model's shapes, with the outline of each text shape.
    pub shapes: SdfShapes,

    /// The edge length of one cell, in meters.
    pub voxel_size: f64,
}

impl CheckedModel {
    /// The model `main` checked for sampling at `resolution`. Errors on the
    /// first failed check.
    pub fn new(main: &SdfMain, resolution: GridResolution) -> Result<Self> {
        let state = main.state();
        let places = collect_places(state);
        let paths = EntryPaths::new(state, &places);

        check_document(state, &places, &paths)?;
        let materials = resolve_materials(state, &paths)?;

        let shapes = SdfShapes::new(main, text_outlines(main, &paths)?);
        check_shape_boxes(state, &shapes, &paths)?;

        let voxel_size = sample_voxel_size(state, &places, &shapes, resolution)?;

        Ok(Self {
            places,
            materials,
            shapes,
            voxel_size,
        })
    }
}
