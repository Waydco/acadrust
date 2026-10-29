//! Some applications store an entity under a dictionary instead of a block
//! (for example a plot-settings entity filed under the named-object
//! dictionary). The DWG writer only walked block records, so such an entity
//! vanished on save.
use acadrust::classes::DxfClass;
use acadrust::entities::{ExtendedEntity, ExtendedEntityData, LayoutPrintConfigData};
use acadrust::objects::{Dictionary, ObjectType};
use acadrust::types::{DxfVersion, Handle};
use acadrust::{CadDocument, DwgReader, DwgWriter, EntityType};
use std::io::Cursor;

fn drawing(version: DxfVersion) -> (CadDocument, Handle, Handle) {
    let mut doc = CadDocument::with_version(version);
    let root = doc.header.named_objects_dict_handle;
    let dict_handle = doc.allocate_handle();
    // As in the files that carry it, the entity's class is registered.
    doc.classes.add_or_update(DxfClass::new_entity(
        "LAYOUTPRINTCONFIG",
        "CAcLayoutPrintConfig",
    ));

    let config = ExtendedEntity {
        common: Default::default(),
        data: ExtendedEntityData::LayoutPrintConfig(LayoutPrintConfigData {
            class_version: 2,
            flag: 0,
            ..Default::default()
        }),
    };
    let entity = doc
        .add_entity(EntityType::Extended(Box::new(config)))
        .unwrap();
    // File it under the dictionary instead of model space.
    if let Some(ms) = doc.block_records.get_mut("*Model_Space") {
        ms.entity_handles.retain(|h| *h != entity);
    }
    doc.get_entity_mut(entity)
        .unwrap()
        .common_mut()
        .owner_handle = dict_handle;

    let mut dictionary = Dictionary::new();
    dictionary.handle = dict_handle;
    dictionary.owner = root;
    dictionary.add_entry("CSD_PLOT_CONFIG", entity);
    doc.objects
        .insert(dict_handle, ObjectType::Dictionary(dictionary));
    if let Some(ObjectType::Dictionary(r)) = doc.objects.get_mut(&root) {
        r.add_entry("CSD_PLOT_CONFIG_DICT", dict_handle);
    }
    (doc, dict_handle, entity)
}

#[test]
fn dictionary_owned_entity_survives_dwg_save() {
    for version in [DxfVersion::AC1015, DxfVersion::AC1018, DxfVersion::AC1032] {
        let (doc, dict_handle, entity) = drawing(version);
        let back = DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
            .read()
            .unwrap();
        let restored = back
            .get_entity(entity)
            .unwrap_or_else(|| panic!("{version:?}: dictionary-owned entity lost"));
        assert!(
            matches!(restored, EntityType::Extended(e) if matches!(e.data, ExtendedEntityData::LayoutPrintConfig(_))),
            "{version:?}: wrong entity type"
        );
        assert_eq!(
            restored.common().owner_handle,
            dict_handle,
            "{version:?}: owner changed"
        );
        assert!(
            !back
                .block_records
                .iter()
                .any(|br| br.entity_handles.contains(&entity)),
            "{version:?}: entity was moved into a block"
        );
    }
}
