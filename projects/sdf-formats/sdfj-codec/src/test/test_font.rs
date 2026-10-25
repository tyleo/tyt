use crate::TestGlyph;

/// A TrueType font of one face named `family` and `full_name` at `weight`
/// holding `glyphs`. A `cap_height` of 0 leaves the OS/2 table without one,
/// and the full name takes Mac Roman.
pub fn test_font(
    family: &str,
    full_name: &str,
    weight: u16,
    cap_height: i16,
    glyphs: &[TestGlyph],
) -> Vec<u8> {
    let count = u16::try_from(glyphs.len() + 1).unwrap();

    // Glyph 0 holds `.notdef`, empty.
    let mut glyf = Vec::new();
    let mut loca = vec![0u32, 0];
    let mut hmtx = vec![0, 0, 0, 0];
    for (_, advance, contours) in glyphs {
        hmtx.extend(advance.to_be_bytes());
        hmtx.extend([0, 0]);

        if !contours.is_empty() {
            let points: Vec<_> = contours.iter().flat_map(|contour| contour.iter()).collect();
            let xs = points.iter().map(|point| point.0);
            let ys = points.iter().map(|point| point.1);

            glyf.extend(i16::try_from(contours.len()).unwrap().to_be_bytes());
            for bound in [
                xs.clone().min().unwrap(),
                ys.clone().min().unwrap(),
                xs.max().unwrap(),
                ys.max().unwrap(),
            ] {
                glyf.extend(bound.to_be_bytes());
            }

            let mut end = 0;
            for contour in *contours {
                end += contour.len();
                glyf.extend(u16::try_from(end - 1).unwrap().to_be_bytes());
            }
            glyf.extend([0, 0]);
            glyf.extend(points.iter().map(|point| u8::from(point.2)));

            // Each coordinate is a 16-bit step from the last point's.
            for axis in [0, 1] {
                let mut last = 0;
                for point in &points {
                    let value = if axis == 0 { point.0 } else { point.1 };
                    glyf.extend((value - last).to_be_bytes());
                    last = value;
                }
            }
        }

        loca.push(u32::try_from(glyf.len()).unwrap());
    }

    let mut cmap = [0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12].to_vec();
    let mut mapped: Vec<_> = (1..)
        .zip(glyphs)
        .map(|(glyph_id, glyph)| (glyph.0, glyph_id))
        .collect();
    mapped.sort();
    cmap.extend([0, 12, 0, 0]);
    cmap.extend(u32::try_from(16 + 12 * mapped.len()).unwrap().to_be_bytes());
    cmap.extend([0, 0, 0, 0]);
    cmap.extend(u32::try_from(mapped.len()).unwrap().to_be_bytes());
    for (character, glyph_id) in mapped {
        cmap.extend(u32::from(character).to_be_bytes());
        cmap.extend(u32::from(character).to_be_bytes());
        cmap.extend(u32::to_be_bytes(glyph_id));
    }

    let mut head = vec![0; 54];
    head[..4].copy_from_slice(&[0, 1, 0, 0]);
    head[12..16].copy_from_slice(&0x5F0F_3CF5u32.to_be_bytes());
    head[18..20].copy_from_slice(&1000u16.to_be_bytes());
    head[50..52].copy_from_slice(&1u16.to_be_bytes());

    let mut hhea = vec![0; 36];
    hhea[..4].copy_from_slice(&[0, 1, 0, 0]);
    hhea[34..36].copy_from_slice(&count.to_be_bytes());

    let mut maxp = vec![0, 0, 0x50, 0];
    maxp.extend(count.to_be_bytes());

    let mut os2 = vec![0; 96];
    os2[..2].copy_from_slice(&2u16.to_be_bytes());
    os2[4..6].copy_from_slice(&weight.to_be_bytes());
    os2[6..8].copy_from_slice(&5u16.to_be_bytes());
    os2[88..90].copy_from_slice(&cap_height.to_be_bytes());

    // The full name takes Mac Roman, the platform 1 encoding 0, and the others
    // UTF-16, the platform 3 encoding 1.
    let names = [(1, family), (4, full_name), (6, full_name)];
    let mut name = vec![0, 0];
    name.extend(u16::try_from(names.len()).unwrap().to_be_bytes());
    name.extend(u16::try_from(6 + 12 * names.len()).unwrap().to_be_bytes());
    let mut storage = Vec::new();
    for (name_id, text) in names {
        let (platform_id, encoding_id, language_id, text) = if name_id == 4 {
            (1, 0, 0, text.bytes().collect())
        } else {
            let text: Vec<_> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
            (3, 1, 0x409, text)
        };
        for field in [
            platform_id,
            encoding_id,
            language_id,
            name_id,
            text.len(),
            storage.len(),
        ] {
            name.extend(u16::try_from(field).unwrap().to_be_bytes());
        }
        storage.extend(text);
    }
    name.extend(storage);

    let loca: Vec<_> = loca.into_iter().flat_map(u32::to_be_bytes).collect();

    // The table records sort by tag.
    let tables = [
        (b"OS/2", os2),
        (b"cmap", cmap),
        (b"glyf", glyf),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"loca", loca),
        (b"maxp", maxp),
        (b"name", name),
    ];
    let mut font = vec![0, 1, 0, 0];
    font.extend(u16::try_from(tables.len()).unwrap().to_be_bytes());
    font.extend([0; 6]);
    let offset = 12 + 16 * tables.len();
    let mut data = Vec::new();
    for (tag, table) in &tables {
        font.extend(*tag);
        font.extend([0; 4]);
        font.extend(u32::try_from(offset + data.len()).unwrap().to_be_bytes());
        font.extend(u32::try_from(table.len()).unwrap().to_be_bytes());
        data.extend(table);
        data.resize(data.len().next_multiple_of(4), 0);
    }
    font.extend(data);
    font
}
