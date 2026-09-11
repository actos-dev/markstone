use markstone_conformance::{Mode, find_cases_dir, load_cases};

#[test]
fn test_rust_binding_conformance_all_77_cases_308_checks() {
    let cases_dir = find_cases_dir().expect("Failed to locate conformance/cases directory");
    let cases = load_cases(&cases_dir, None).expect("Failed to load conformance cases");
    assert_eq!(cases.len(), 77, "Expected 77 conformance cases");

    let mut checks_count = 0;
    for case in &cases {
        for mode in Mode::ALL {
            let actual: Vec<u8> = match mode {
                Mode::GenericHtml => markstone::to_html(&case.input)
                    .unwrap_or_else(|e| panic!("markstone::to_html failed on {}: {e:?}", case.name))
                    .into_bytes(),
                Mode::GenericAst => markstone::to_ast(&case.input)
                    .unwrap_or_else(|e| panic!("markstone::to_ast failed on {}: {e:?}", case.name))
                    .into_bytes(),
                Mode::ActosHtml => markstone::actos::to_html(&case.input)
                    .unwrap_or_else(|e| {
                        panic!("markstone::actos::to_html failed on {}: {e:?}", case.name)
                    })
                    .into_bytes(),
                Mode::ActosAst => markstone::actos::to_ast(&case.input)
                    .unwrap_or_else(|e| {
                        panic!("markstone::actos::to_ast failed on {}: {e:?}", case.name)
                    })
                    .into_bytes(),
            };

            let expected = case.expected.get(&mode).unwrap_or_else(|| {
                panic!("Missing expected golden for case {} mode {mode}", case.name)
            });

            if &actual != expected {
                panic!(
                    "Conformance mismatch in case {} mode {mode}\nExpected:\n{}\nActual:\n{}",
                    case.name,
                    String::from_utf8_lossy(expected),
                    String::from_utf8_lossy(&actual)
                );
            }
            checks_count += 1;
        }
    }

    assert_eq!(checks_count, 77 * 4); // 308 checks
}

#[test]
fn test_rust_binding_bytes_api() {
    let input = "# Test\n\nHello @alice and #tag";
    let bytes = input.as_bytes();

    let generic_html = markstone::to_html_bytes(bytes).unwrap();
    let generic_ast = markstone::to_ast_bytes(bytes).unwrap();
    let actos_html = markstone::actos::to_html_bytes(bytes).unwrap();
    let actos_ast = markstone::actos::to_ast_bytes(bytes).unwrap();

    assert_eq!(generic_html, markstone::to_html(input).unwrap());
    assert_eq!(generic_ast, markstone::to_ast(input).unwrap());
    assert_eq!(actos_html, markstone::actos::to_html(input).unwrap());
    assert_eq!(actos_ast, markstone::actos::to_ast(input).unwrap());
}

#[test]
fn test_rust_binding_constants_and_errors() {
    assert_eq!(markstone::AST_SCHEMA_VERSION, 1);

    // Invalid UTF-8
    let invalid_utf8 = [0xff, 0xfe, 0xfd];
    let err = markstone::to_html_bytes(&invalid_utf8).unwrap_err();
    assert!(matches!(err, markstone::MarkstoneError::InvalidUtf8));
    let err_actos = markstone::actos::to_html_bytes(&invalid_utf8).unwrap_err();
    assert!(matches!(err_actos, markstone::MarkstoneError::InvalidUtf8));

    let err_ast = markstone::to_ast_bytes(&invalid_utf8).unwrap_err();
    assert!(matches!(err_ast, markstone::MarkstoneError::InvalidUtf8));
    let err_actos_ast = markstone::actos::to_ast_bytes(&invalid_utf8).unwrap_err();
    assert!(matches!(
        err_actos_ast,
        markstone::MarkstoneError::InvalidUtf8
    ));
}
