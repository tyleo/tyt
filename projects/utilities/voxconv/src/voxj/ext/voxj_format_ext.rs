use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, push_slot, push_slots},
    voxj::{
        Voxj, VoxjSerialization, VoxjWriteFormat,
        ext::{
            CompositeVoxExt, InertVoxExt, composite_vox_ext_from_voxj_vox_ext,
            voxj_vox_ext_from_ext,
        },
    },
};
use voxcore::{TakenExt, VoxMapEntry};
use voxj_voxcore::{
    VoxjVoxExt,
    codec::{from_voxj_bytes, to_voxj_bytes, to_voxj_pretty_bytes, to_voxjz_bytes},
};

impl FormatExt for Voxj {
    /// The `ext` block decodes into a
    /// [`CompositeVoxExt`](crate::voxj::ext::CompositeVoxExt), with each
    /// installed format's slot as that format's ext.
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        let TakenExt { main, ext } =
            from_voxj_bytes(dependencies.voxj(), VoxDocumentFile::single_bytes(files)?)?.take_ext();

        Ok(box_ext(
            main.put_ext(composite_vox_ext_from_voxj_vox_ext(ext)?),
        ))
    }

    /// The ext encodes as the `ext` block.
    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &VoxjWriteFormat,
        main: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let ext = voxj_vox_ext_from_ext(main.ext().as_ref())?;

        let main = main.take_ext().main.put_ext(ext);

        let dependencies = dependencies.voxj();

        let bytes = match options.serialization {
            VoxjSerialization::Compact => to_voxj_bytes(dependencies, &main, &options.options)?,
            VoxjSerialization::Pretty => {
                to_voxj_pretty_bytes(dependencies, &main, &options.options)?
            }
            VoxjSerialization::Zip => to_voxjz_bytes(dependencies, &main, &options.options)?,
        };

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    /// Voxel Json's exts are the block's forms: a [`VoxjVoxExt`] encodes as its
    /// slots, a [`CompositeVoxExt`] as every slot its exts encode to, and an
    /// [`InertVoxExt`] as the slot it kept.
    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        if let Some(voxj) = ext.downcast_ref::<VoxjVoxExt>() {
            for slot in voxj.slots() {
                push_slot(slots, slot.clone())?;
            }

            return Ok(true);
        }

        if let Some(composite) = ext.downcast_ref::<CompositeVoxExt>() {
            for ext in &composite.exts {
                push_slots(ext.as_ref(), slots)?;
            }

            return Ok(true);
        }

        if let Some(inert) = ext.downcast_ref::<InertVoxExt>() {
            push_slot(
                slots,
                VoxMapEntry {
                    key: inert.key.clone(),
                    value: inert.value.clone(),
                },
            )?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Voxel Json has no slot of its own. A slot under no format's key stays
    /// inert.
    fn decode_slot(_slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        Ok(None)
    }
}
