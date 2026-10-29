//! Reading a RAY or XLINE must keep its stored direction exactly. Normalizing
//! on read changes the last bits of nearly-unit vectors, so an unchanged
//! drawing no longer round-trips identically.
use acadrust::entities::{Ray, XLine};
use acadrust::types::{DxfVersion, Vector3};
use acadrust::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

// Taken from a real drawing: unit length to within rounding, but not the
// value `normalize()` produces.
const DIR: Vector3 = Vector3 {
    x: 0.8208410212999242,
    y: 0.5711567365892635,
    z: 0.0,
};
const BASE: Vector3 = Vector3 {
    x: 11211.629932684707,
    y: 5441.204240095531,
    z: 0.0,
};

fn drawing(version: DxfVersion) -> CadDocument {
    let mut doc = CadDocument::new();
    doc.version = version;
    let mut ray = Ray::new(BASE, DIR);
    ray.direction = DIR;
    let mut xline = XLine::new(BASE, DIR);
    xline.direction = DIR;
    doc.add_entity(EntityType::Ray(ray)).unwrap();
    doc.add_entity(EntityType::XLine(xline)).unwrap();
    doc
}

fn directions(doc: &CadDocument) -> (Vector3, Vector3) {
    let ray = doc.entities().find_map(|e| match e {
        EntityType::Ray(r) => Some(r.direction),
        _ => None,
    });
    let xline = doc.entities().find_map(|e| match e {
        EntityType::XLine(x) => Some(x.direction),
        _ => None,
    });
    (ray.unwrap(), xline.unwrap())
}

fn assert_exact(v: Vector3, what: &str) {
    assert_eq!(
        (v.x.to_bits(), v.y.to_bits(), v.z.to_bits()),
        (DIR.x.to_bits(), DIR.y.to_bits(), DIR.z.to_bits()),
        "{what}: {v:?}"
    );
}

#[test]
fn dwg_keeps_ray_and_xline_direction_bits() {
    assert_ne!(
        DIR.normalize().x.to_bits(),
        DIR.x.to_bits(),
        "test vector must be affected by normalize"
    );
    for version in [DxfVersion::AC1015, DxfVersion::AC1018, DxfVersion::AC1032] {
        let doc = drawing(version);
        let back = DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
            .read()
            .unwrap();
        let (r, x) = directions(&back);
        assert_exact(r, &format!("{version:?} ray"));
        assert_exact(x, &format!("{version:?} xline"));
    }
}

#[test]
fn dxf_keeps_ray_and_xline_direction_bits() {
    let doc = drawing(DxfVersion::AC1032);
    let back = DxfReader::from_reader(Cursor::new(DxfWriter::new(&doc).write_to_vec().unwrap()))
        .unwrap()
        .read()
        .unwrap();
    let (r, x) = directions(&back);
    assert_exact(r, "dxf ray");
    assert_exact(x, "dxf xline");
}
