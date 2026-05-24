#[test]
fn rust_methods_emit_self_receivers() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Rectangle {
    numerus width = 1
    numerus height = 1

    functio area() → numerus {
        redde ego.width * ego.height
    }
}

genus Counter {
    numerus count = 0

    functio increment() {
        ego.count ← ego.count + 1
    }

    functio getValue() → numerus {
        redde ego.count
    }
}

incipit {
    fixum _ rect ← Rectangle { width = 10, height = 5 }
    nota rect.area()
    varia _ counter ← Counter { count = 0 }
    counter.increment()
    nota counter.getValue()
}
"#;

    let result = compiler.compile_str("rust-method-receivers.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("fn area(&self) -> i64"));
    assert!(rust.code.contains("return self.width * self.height;"));
    assert!(rust.code.contains("fn increment(&mut self) -> ()"));
    assert!(rust.code.contains("self.count = self.count + 1;"));
    assert!(rust.code.contains("fn getValue(&self) -> i64"));
    assert!(!rust.code.contains("Rectangle.width"));
    assert!(!rust.code.contains("Counter.count"));
}

#[test]
fn typed_struct_construction_calls_creo_hook() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Circle {
    numerus radius = 1
    fractus area = 0

    functio creo() {
        ego.area ← 3.14159 * ego.radius * ego.radius
    }
}

incipit {
    fixum _ circle ← Circle { radius = 5 }
    nota circle.area
}
"#;

    let result = compiler.compile_str("creo-hook-rust.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("pub area: f64,"));
    assert!(rust.code.contains("let circle: Circle = {"));
    assert!(rust.code.contains("let mut __faber_struct_"));
    assert!(rust.code.contains("area: 0.0,"));
    assert!(rust.code.contains(".creo();"));
    assert!(rust.code.contains("return self.area;") || rust.code.contains("circle.area"));
}

#[test]
fn empty_typed_constructor_uses_field_defaults() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Counter {
    numerus count = 0
}

incipit {
    varia _ counter ← Counter {}
    nota counter.count
}
"#;

    let result = compiler.compile_str("empty-typed-constructor.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust.code.contains("let mut counter: Counter = Counter {"));
    assert!(rust.code.contains("count: 0,"));
    assert!(!rust.code.contains("Counter;"));
}

#[test]
fn self_returning_methods_use_mutable_receiver_return() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
genus Calculator {
    numerus value = 0

    functio setValue(numerus n) → Calculator {
        ego.value ← n
        redde ego
    }

    functio double() → Calculator {
        ego.value ← ego.value * 2
        redde ego
    }

    functio getResult() → numerus {
        redde ego.value
    }
}

incipit {
    varia _ calc ← Calculator {}
    fixum _ result ← calc.setValue(5).double().getResult()
    nota result
}
"#;

    let result = compiler.compile_str("self-return-method.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(rust
        .code
        .contains("fn setValue(&mut self, n: i64) -> &mut Calculator"));
    assert!(rust
        .code
        .contains("fn double(&mut self) -> &mut Calculator"));
    assert!(rust.code.contains("return self;"));
    assert!(rust
        .code
        .contains("let result: i64 = calc.setValue(5).double().getResult();"));
}
