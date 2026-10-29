//! A LEADER read from a DWG file whose stored origin is (0,0,0) must keep it.
//! The writer replaced a zero origin with the first vertex, which is a useful
//! default for leaders built in code or read from DXF (where the origin is not
//! stored), but it changed real AutoCAD drawings on every save.
use acadrust::entities::Leader;
use acadrust::types::{DxfVersion, Vector3};
use acadrust::{CadDocument, DwgReader, DwgWriter, EntityType};
use std::io::Cursor;

fn leader(doc: &CadDocument) -> &Leader {
    doc.entities()
        .find_map(|e| match e {
            EntityType::Leader(l) => Some(l),
            _ => None,
        })
        .unwrap()
}

fn read_back(doc: &CadDocument) -> CadDocument {
    DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(doc).unwrap()))
        .read()
        .unwrap()
}

#[test]
fn zero_origin_read_from_dwg_survives_save() {
    for version in [DxfVersion::AC1015, DxfVersion::AC1032] {
        let mut doc = CadDocument::with_version(version);
        let mut l = Leader::new();
        l.vertices = vec![Vector3::new(2.5, 1.9, 0.0), Vector3::new(4.0, 3.0, 0.0)];
        doc.add_entity(EntityType::Leader(l)).unwrap();

        // Built in code: the first vertex stands in for the missing origin.
        let mut from_dwg = read_back(&doc);
        assert_eq!(
            leader(&from_dwg).origin,
            Vector3::new(2.5, 1.9, 0.0),
            "{version:?}"
        );

        // Now a DWG drawing whose leader stores a zero origin, as AutoCAD does.
        for e in from_dwg.entities_mut() {
            if let EntityType::Leader(l) = e {
                l.origin = Vector3::ZERO;
            }
        }
        let again = read_back(&from_dwg);
        assert_eq!(
            leader(&again).origin,
            Vector3::ZERO,
            "{version:?}: stored origin replaced"
        );
    }
}
