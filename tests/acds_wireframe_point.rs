//! R2013+ (AcDs-backed) solids and regions read from a file keep the
//! wireframe reference point that file stored. Dropping it on save makes the
//! reader re-derive a slightly different point and clears the
//! point-present flag, so an unchanged drawing no longer round-trips.
use acadrust::entities::acis::primitives::build_box;
use acadrust::entities::solid3d::{Region, Solid3D};
use acadrust::entities::EntityType;
use acadrust::types::{DxfVersion, Vector3};
use acadrust::{CadDocument, DwgReader, DwgWriter};
use std::io::Cursor;

// A stored point that is not the exact value the reader would derive.
const POINT: Vector3 = Vector3 {
    x: 1.0000000000000013,
    y: 1.4999999999999991,
    z: 2.0000000000000013,
};

fn drawing(version: DxfVersion, from_file: bool) -> CadDocument {
    let mut doc = CadDocument::with_version(version);
    let sat = build_box([0.0, 0.0, 0.0], 2.0, 3.0, 4.0).to_sat_string();
    let mut solid = Solid3D::from_sat(&sat);
    let mut region = Region::from_sat(&sat);
    for (acis, point) in [
        (&mut solid.acis_data, &mut solid.point_of_reference),
        (&mut region.acis_data, &mut region.point_of_reference),
    ] {
        acis.wireframe_data_present = true;
        acis.wireframe_isolines = 4;
        acis.wireframe_isoline_present = true;
        acis.wireframe_point_present = from_file;
        if from_file {
            *point = POINT;
        }
    }
    doc.add_entity(EntityType::Solid3D(solid)).unwrap();
    doc.add_entity(EntityType::Region(region)).unwrap();
    doc
}

fn read_back(doc: &CadDocument) -> CadDocument {
    DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(doc).unwrap()))
        .read()
        .unwrap()
}

fn points(doc: &CadDocument) -> Vec<(&'static str, Vector3, bool)> {
    doc.entities()
        .filter_map(|e| match e {
            EntityType::Solid3D(s) => Some((
                "solid",
                s.point_of_reference,
                s.acis_data.wireframe_point_present,
            )),
            EntityType::Region(r) => Some((
                "region",
                r.point_of_reference,
                r.acis_data.wireframe_point_present,
            )),
            _ => None,
        })
        .collect()
}

fn bits(v: Vector3) -> (u64, u64, u64) {
    (v.x.to_bits(), v.y.to_bits(), v.z.to_bits())
}

#[test]
fn stored_wireframe_point_survives_acds_save() {
    for version in [DxfVersion::AC1027, DxfVersion::AC1032] {
        let back = read_back(&drawing(version, true));
        let found = points(&back);
        assert_eq!(found.len(), 2, "{version:?}");
        for (kind, point, present) in found {
            assert!(present, "{version:?} {kind}: point-present flag lost");
            assert_eq!(bits(point), bits(POINT), "{version:?} {kind}: {point:?}");
        }
    }
}

#[test]
fn application_built_bodies_still_omit_the_point() {
    for version in [DxfVersion::AC1027, DxfVersion::AC1032] {
        let back = read_back(&drawing(version, false));
        for (kind, _, present) in points(&back) {
            assert!(
                !present,
                "{version:?} {kind}: a synthesized point was written"
            );
        }
    }
}
