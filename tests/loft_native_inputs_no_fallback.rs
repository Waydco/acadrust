//! A lofted surface whose cross sections are written natively in the DWG
//! record must not also get a private fallback dictionary. The native group
//! round-trips by itself, and the extra dictionary changes an unchanged
//! drawing on every save.
use acadrust::entities::acis::primitives::build_box;
use acadrust::entities::solid3d::AcisData;
use acadrust::entities::surface::{Surface, SurfaceData, SurfaceKind};
use acadrust::entities::{Circle, EmbeddedEntity, EntityType};
use acadrust::objects::ObjectType;
use acadrust::types::{DxfVersion, Vector3};
use acadrust::{CadDocument, DwgReader, DwgWriter};
use std::io::Cursor;

fn drawing(version: DxfVersion) -> CadDocument {
    let mut doc = CadDocument::with_version(version);
    let mut surface = Surface::new(SurfaceKind::Lofted);
    surface.acis_data =
        AcisData::from_sat(&build_box([0.0, 0.0, 0.0], 2.0, 2.0, 2.0).to_sat_string());
    if let SurfaceData::Lofted {
        cross_section_entities,
        ..
    } = &mut surface.surface_data
    {
        cross_section_entities.push(EmbeddedEntity::Circle(Circle::from_center_radius(
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        )));
        cross_section_entities.push(EmbeddedEntity::Circle(Circle::from_center_radius(
            Vector3::new(0.0, 0.0, 2.0),
            1.5,
        )));
    }
    doc.add_entity(EntityType::Surface(Box::new(surface)))
        .unwrap();
    doc
}

#[test]
fn natively_written_loft_inputs_add_no_fallback_dictionary() {
    for version in [DxfVersion::AC1018, DxfVersion::AC1024, DxfVersion::AC1032] {
        let doc = drawing(version);
        let back = DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
            .read()
            .unwrap();
        let surface = back
            .entities()
            .find_map(|e| match e {
                EntityType::Surface(s) => Some(s),
                _ => None,
            })
            .unwrap();
        let SurfaceData::Lofted {
            cross_section_entities,
            ..
        } = &surface.surface_data
        else {
            panic!("{version:?}: not a lofted surface");
        };
        assert_eq!(
            cross_section_entities.len(),
            2,
            "{version:?}: native cross sections lost"
        );
        assert_eq!(
            surface.common.xdictionary_handle, None,
            "{version:?}: fallback dictionary added"
        );
        assert!(
            !back.objects.values().any(
                |o| matches!(o, ObjectType::XRecord(r) if r.name.starts_with("CADCODEC_LOFT"))
            ),
            "{version:?}: fallback record written"
        );
    }
}
