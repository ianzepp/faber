use super::{
    parse_wat_import_sites, probe_wat_instantiation, WasmInstantiationBucket, WasmInstantiationProbe,
};

#[test]
fn parses_wat_import_sites() {
    let wat = r#"
(module
    (import "faber_diag" "nota_i32" (func $__faber_diag_nota_i32 (param i32)))
    (import "faber_runtime" "append" (func $__faber_runtime_append (param i32 i32)))
)
"#;
    let imports = parse_wat_import_sites(wat);
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].module, "faber_diag");
    assert_eq!(imports[0].name, "nota_i32");
    assert_eq!(imports[1].module, "faber_runtime");
    assert_eq!(imports[1].name, "append");
}

#[test]
fn classifies_missing_import_for_stubless_host() {
    let wat = r#"
(module
    (import "faber_diag" "nota_i32" (func $__faber_diag_nota_i32 (param i32)))
    (func $main (export "main")
        (call $__faber_diag_nota_i32 (i32.const 0))
    )
)
"#;
    let WasmInstantiationProbe { bucket, reason, imports } = probe_wat_instantiation(wat);
    assert_eq!(bucket, WasmInstantiationBucket::MissingImport);
    assert_eq!(imports.len(), 1);
    assert!(reason.contains("faber_diag::nota_i32") || reason.contains("unknown import"));
}

#[test]
fn classifies_import_free_module_as_instantiate_valid() {
    let wat = r#"
(module
    (func $main (export "main")
        (return)
    )
)
"#;
    let WasmInstantiationProbe { bucket, reason, imports } = probe_wat_instantiation(wat);
    assert_eq!(bucket, WasmInstantiationBucket::InstantiateValid);
    assert!(imports.is_empty());
    assert!(reason.contains("no imports"));
}