use std::{collections::HashSet, env, fs, path::PathBuf};

use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{
    Decl, EsVersion, Module, ModuleDecl, ModuleExportName, ModuleItem, ObjectPatProp, Pat,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

const SOURCE_FILE: &str = "src/omni-zenfs.ts";
const WRAPPER_FILE: &str = "omni-zenfs.js";

fn main() {
    println!("cargo:rerun-if-changed={SOURCE_FILE}");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(manifest_dir.join(SOURCE_FILE))
        .expect("failed to read src/omni-zenfs.ts");
    let exports = collect_exports(&source, SOURCE_FILE)
        .unwrap_or_else(|e| panic!("failed to parse {SOURCE_FILE}: {e}"));

    assert!(
        !exports.is_empty(),
        "no exported symbols found in src/omni-zenfs.ts"
    );

    let wrapper = format!(
        "export {{ {} }} from '/omni-zenfs.js';\n",
        exports.join(", ")
    );
    let wrapper_path = manifest_dir.join(WRAPPER_FILE);

    let current = fs::read_to_string(&wrapper_path).ok();
    if current.as_deref() != Some(wrapper.as_str()) {
        fs::write(wrapper_path, wrapper).expect("failed to write omni-zenfs.js wrapper");
    }
}

fn collect_exports(source: &str, filename: &str) -> Result<Vec<String>, String> {
    let module = parse_typescript_module(source, filename)?;
    let mut exports = Vec::new();
    let mut seen = HashSet::new();

    for item in module.body {
        let ModuleItem::ModuleDecl(decl) = item else {
            continue;
        };

        for symbol in collect_from_module_decl(&decl) {
            if seen.insert(symbol.clone()) {
                exports.push(symbol);
            }
        }
    }

    Ok(exports)
}

fn parse_typescript_module(source: &str, filename: &str) -> Result<Module, String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(filename.to_owned()).into(),
        source.to_owned(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: false,
        }),
        EsVersion::Es2022,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().map_err(|e| format!("{e:?}"))?;

    let parser_errors = parser.take_errors();
    if let Some(first) = parser_errors.into_iter().next() {
        return Err(format!("{first:?}"));
    }

    Ok(module)
}

fn collect_from_module_decl(decl: &ModuleDecl) -> Vec<String> {
    match decl {
        ModuleDecl::ExportDecl(export_decl) => collect_from_decl(&export_decl.decl),
        ModuleDecl::ExportNamed(named) => {
            if named.type_only {
                return Vec::new();
            }

            named
                .specifiers
                .iter()
                .filter_map(|specifier| match specifier {
                    swc_ecma_ast::ExportSpecifier::Named(named_specifier) => {
                        if named_specifier.is_type_only {
                            return None;
                        }
                        named_specifier
                            .exported
                            .as_ref()
                            .and_then(module_export_name_to_string)
                            .or_else(|| module_export_name_to_string(&named_specifier.orig))
                    }
                    swc_ecma_ast::ExportSpecifier::Default(default_specifier) => {
                        Some(default_specifier.exported.sym.to_string())
                    }
                    swc_ecma_ast::ExportSpecifier::Namespace(namespace_specifier) => {
                        module_export_name_to_string(&namespace_specifier.name)
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn collect_from_decl(decl: &Decl) -> Vec<String> {
    match decl {
        Decl::Fn(function_decl) => vec![function_decl.ident.sym.to_string()],
        Decl::Class(class_decl) => vec![class_decl.ident.sym.to_string()],
        Decl::Var(variable_decl) => {
            let mut symbols = Vec::new();
            for declaration in &variable_decl.decls {
                collect_from_pat(&declaration.name, &mut symbols);
            }
            symbols
        }
        _ => Vec::new(),
    }
}

fn collect_from_pat(pattern: &Pat, symbols: &mut Vec<String>) {
    match pattern {
        Pat::Ident(binding_ident) => symbols.push(binding_ident.id.sym.to_string()),
        Pat::Array(array_pattern) => {
            for element in array_pattern.elems.iter().flatten() {
                collect_from_pat(element, symbols);
            }
        }
        Pat::Object(object_pattern) => {
            for property in &object_pattern.props {
                match property {
                    ObjectPatProp::Assign(assign_pattern) => {
                        symbols.push(assign_pattern.key.sym.to_string())
                    }
                    ObjectPatProp::KeyValue(key_value_pattern) => {
                        collect_from_pat(&key_value_pattern.value, symbols)
                    }
                    ObjectPatProp::Rest(rest_pattern) => {
                        collect_from_pat(&rest_pattern.arg, symbols)
                    }
                }
            }
        }
        Pat::Assign(assign_pattern) => collect_from_pat(&assign_pattern.left, symbols),
        Pat::Rest(rest_pattern) => collect_from_pat(&rest_pattern.arg, symbols),
        _ => {}
    }
}

fn module_export_name_to_string(name: &ModuleExportName) -> Option<String> {
    match name {
        ModuleExportName::Ident(ident) => Some(ident.sym.to_string()),
        ModuleExportName::Str(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_exports;

    #[test]
    fn parses_function_exports() {
        let source = r#"
            export async function init() {}
            export function readFile(path: string) {}
        "#;

        assert_eq!(
            collect_exports(source, "test.ts").unwrap(),
            vec!["init", "readFile"]
        );
    }

    #[test]
    fn parses_const_and_class_exports() {
        let source = r#"
            export const one = 1;
            export let two = 2;
            export var three = 3;
            export class Store {}
        "#;

        assert_eq!(
            collect_exports(source, "test.ts").unwrap(),
            vec!["one", "two", "three", "Store"]
        );
    }

    #[test]
    fn parses_export_blocks_and_aliases() {
        let source = r#"
            const open = 1;
            const close = 2;
            const mkdir = 3;
            export {
              open,
              close as closeFile,
              type Internal,
              mkdir,
            };
        "#;

        assert_eq!(
            collect_exports(source, "test.ts").unwrap(),
            vec!["open", "closeFile", "mkdir"]
        );
    }

    #[test]
    fn ignores_type_only_and_deduplicates() {
        let source = r#"
            export function init() {}
            export { init, init as initAlias, type Internal };
        "#;

        assert_eq!(
            collect_exports(source, "test.ts").unwrap(),
            vec!["init", "initAlias"]
        );
    }

    #[test]
    fn parses_destructured_variable_exports() {
        let source = r#"
            export const { a, b: c } = source;
            export const [d, e] = list;
        "#;

        assert_eq!(
            collect_exports(source, "test.ts").unwrap(),
            vec!["a", "c", "d", "e"]
        );
    }
}
