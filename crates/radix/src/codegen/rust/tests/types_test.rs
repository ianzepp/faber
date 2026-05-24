use super::*;

#[test]
fn type_to_rust_covers_composite_and_special_cases() {
    let mut interner = Interner::new();
    let sym_t = interner.intern("T");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let fractus = types.primitive(Primitive::Fractus);
    let regex = types.primitive(Primitive::Regex);

    let struct_ty = types.intern(Type::Struct(DefId(100)));
    let enum_ty = types.intern(Type::Enum(DefId(101)));
    let iface_ty = types.intern(Type::Interface(DefId(102)));
    let alias_ty = types.intern(Type::Alias(DefId(103), numerus));
    let array_ty = types.array(numerus);
    let map_ty = types.map(textus, numerus);
    let set_ty = types.set(fractus);
    let option_ty = types.option(numerus);
    let ref_ty = types.reference(Mutability::Immutable, numerus);
    let mut_ref_ty = types.reference(Mutability::Mutable, numerus);
    let param_ty = types.intern(Type::Param(sym_t));
    let applied_ty = types.intern(Type::Applied(struct_ty, vec![numerus]));
    let infer_ty = types.intern(Type::Infer(InferVar(1)));
    let union_empty_ty = types.intern(Type::Union(Vec::new()));
    let union_ty = types.intern(Type::Union(vec![numerus, textus]));
    let error_ty = types.intern(Type::Error);
    let sync_fn_ty = types.function(FuncSig {
        params: vec![ParamType { ty: numerus, mode: ParamMode::Owned, optional: false }],
        ret: textus,
        err: None,
        is_async: false,
        is_generator: false,
    });
    let async_fn_ty = types.function(FuncSig {
        params: vec![ParamType { ty: numerus, mode: ParamMode::Owned, optional: false }],
        ret: textus,
        err: None,
        is_async: true,
        is_generator: false,
    });

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(790),
                def_id: DefId(100),
                kind: HirItemKind::Struct(HirStruct {
                    name: interner.intern("Structum"),
                    type_params: Vec::new(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    extends: None,
                    implements: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(791),
                def_id: DefId(101),
                kind: HirItemKind::Enum(HirEnum {
                    name: interner.intern("Enumeratio"),
                    type_params: Vec::new(),
                    variants: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(792),
                def_id: DefId(102),
                kind: HirItemKind::Interface(HirInterface {
                    name: interner.intern("Officium"),
                    type_params: Vec::new(),
                    methods: Vec::new(),
                }),
                span: span(),
            },
        ],
        entry: None,
    };
    let codegen = super::super::RustCodegen::new(&program, &interner);

    assert_eq!(super::super::types::type_to_rust(&codegen, numerus, &types), "i64");
    assert_eq!(super::super::types::type_to_rust(&codegen, regex, &types), "regex::Regex");
    assert_eq!(super::super::types::type_to_rust(&codegen, array_ty, &types), "Vec<i64>");
    assert_eq!(
        super::super::types::type_to_rust(&codegen, map_ty, &types),
        "HashMap<String, i64>"
    );
    assert_eq!(super::super::types::type_to_rust(&codegen, set_ty, &types), "HashSet<f64>");
    assert_eq!(super::super::types::type_to_rust(&codegen, option_ty, &types), "Option<i64>");
    assert_eq!(super::super::types::type_to_rust(&codegen, ref_ty, &types), "&i64");
    assert_eq!(super::super::types::type_to_rust(&codegen, mut_ref_ty, &types), "&mut i64");
    assert_eq!(super::super::types::type_to_rust(&codegen, struct_ty, &types), "Structum");
    assert_eq!(super::super::types::type_to_rust(&codegen, enum_ty, &types), "Enumeratio");
    assert_eq!(super::super::types::type_to_rust(&codegen, iface_ty, &types), "dyn Officium");
    assert_eq!(super::super::types::type_to_rust(&codegen, alias_ty, &types), "i64");
    assert_eq!(
        super::super::types::type_to_rust(&codegen, sync_fn_ty, &types),
        "fn(i64) -> String"
    );
    assert_eq!(
        super::super::types::type_to_rust(&codegen, async_fn_ty, &types),
        "impl Future<Output = String>"
    );
    assert_eq!(super::super::types::type_to_rust(&codegen, param_ty, &types), "T");
    assert_eq!(super::super::types::type_to_rust(&codegen, applied_ty, &types), "Structum<i64>");
    assert_eq!(super::super::types::type_to_rust(&codegen, infer_ty, &types), "_");
    assert_eq!(super::super::types::type_to_rust(&codegen, union_empty_ty, &types), "!");
    assert_eq!(super::super::types::type_to_rust(&codegen, union_ty, &types), "FaberValue");
    assert_eq!(super::super::types::type_to_rust(&codegen, error_ty, &types), "/* error */");
}

#[test]
fn valor_type_renders_to_norma_datum_valor_and_supports_si_valor() {
    let interner = Interner::new();
    let prog = HirProgram { items: vec![], entry: None };
    let codegen = super::super::RustCodegen::new(&prog, &interner);

    let mut types = TypeTable::new();
    let valor = types.primitive(Primitive::Valor);
    let option_valor = types.intern(Type::Option(valor));

    assert_eq!(
        super::super::types::type_to_rust(&codegen, valor, &types),
        "norma::datum::Valor"
    );
    assert_eq!(
        super::super::types::type_to_rust(&codegen, option_valor, &types),
        "Option<norma::datum::Valor>"
    );

    // Explicitly prove we do not fall back to Box<dyn Any> for the data-format type
    let rendered = super::super::types::type_to_rust(&codegen, valor, &types);
    assert!(!rendered.contains("Any"), "valor must not render as Box<dyn Any>: {rendered}");
}
