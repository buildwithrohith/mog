use std::collections::{BTreeMap, BTreeSet};

use cell_types::SheetId;
use compute_parser::{ASTNode, CellRefResolver, parse_formula};
use formula_types::CellRef;
use quick_xml::Reader;
use quick_xml::events::Event;
use value_types::CellError;

use crate::domain::content_types::read::ContentTypes;
use crate::domain::names::DefinedNames;
use crate::domain::workbook::read as workbook;
use crate::zip::{XlsxArchive, ZipError};

/// A reason dependency-closure admission must fail closed for a formula owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyBlocker {
    MalformedFormula,
    UnresolvedSheetReference,
    ExternalReference,
    DynamicIndirect,
    UnsupportedStructuredReference,
    UnresolvedNamedRange,
    CyclicNamedRange,
}

/// Defined-name inventory retained so prospective formulas can be analyzed
/// against the same workbook and sheet scopes as imported formulas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDefinedName {
    pub name: String,
    pub refers_to: String,
    /// OOXML `localSheetId`, which is the zero-based workbook-order index.
    pub scope_workbook_index: Option<u32>,
}

/// Sheet precedents and fail-closed reasons for one formula owner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SheetDependencyAnalysis {
    pub precedents: BTreeSet<u32>,
    pub blockers: BTreeSet<DependencyBlocker>,
}

/// Compact workbook-wide formula dependency inventory keyed only by stable
/// workbook-order indices. No worksheet cell payloads are retained.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SheetDependencyManifest {
    pub sheet_names: BTreeMap<u32, String>,
    pub precedents_by_dependent: BTreeMap<u32, BTreeSet<u32>>,
    pub blockers_by_dependent: BTreeMap<u32, BTreeSet<DependencyBlocker>>,
    pub defined_names: Vec<DependencyDefinedName>,
}

impl SheetDependencyManifest {
    /// Analyze a formula typed after import using this manifest's exact sheet
    /// and defined-name inventory.
    #[must_use]
    pub fn analyze_formula(
        &self,
        owner_workbook_index: u32,
        formula: &str,
    ) -> SheetDependencyAnalysis {
        Analyzer::new(self, owner_workbook_index).analyze(formula)
    }
}

struct WorkbookIndexResolver<'a> {
    manifest: &'a SheetDependencyManifest,
    current: u32,
}

impl CellRefResolver for WorkbookIndexResolver<'_> {
    fn resolve(&self, sheet: &SheetId, row: u32, col: u32) -> CellRef {
        CellRef::Positional {
            sheet: *sheet,
            row,
            col,
        }
    }

    fn resolve_sheet_name(&self, name: &str) -> Option<SheetId> {
        self.manifest
            .sheet_names
            .iter()
            .find(|(_, candidate)| candidate.eq_ignore_ascii_case(name))
            .map(|(index, _)| SheetId::from_raw(u128::from(*index)))
    }

    fn current_sheet(&self) -> SheetId {
        SheetId::from_raw(u128::from(self.current))
    }
}

struct Analyzer<'a> {
    manifest: &'a SheetDependencyManifest,
    owner: u32,
    result: SheetDependencyAnalysis,
    name_stack: Vec<(Option<u32>, String)>,
}

impl<'a> Analyzer<'a> {
    fn new(manifest: &'a SheetDependencyManifest, owner: u32) -> Self {
        Self {
            manifest,
            owner,
            result: SheetDependencyAnalysis::default(),
            name_stack: Vec::new(),
        }
    }

    fn analyze(mut self, formula: &str) -> SheetDependencyAnalysis {
        if !self.manifest.sheet_names.contains_key(&self.owner) {
            self.result
                .blockers
                .insert(DependencyBlocker::UnresolvedSheetReference);
            return self.result;
        }
        self.analyze_expression(self.owner, formula, &BTreeSet::new());
        self.result.precedents.remove(&self.owner);
        self.result
    }

    fn analyze_expression(
        &mut self,
        current_sheet: u32,
        expression: &str,
        locals: &BTreeSet<String>,
    ) {
        let resolver = WorkbookIndexResolver {
            manifest: self.manifest,
            current: current_sheet,
        };
        match parse_formula(expression, Some(&resolver)) {
            Ok(ast) => self.walk(current_sheet, &ast.node, locals),
            Err(_) => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::MalformedFormula);
            }
        }
    }

    fn walk(&mut self, current_sheet: u32, node: &ASTNode, locals: &BTreeSet<String>) {
        match node {
            ASTNode::SheetRef { sheet, inner } => {
                let index = sheet.as_u128() as u32;
                self.result.precedents.insert(index);
                self.walk(index, inner, locals);
            }
            ASTNode::ThreeDRef {
                start_sheet,
                end_sheet,
                inner,
            } => {
                let start = start_sheet.as_u128() as u32;
                let end = end_sheet.as_u128() as u32;
                let (first, last) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                for index in first..=last {
                    if self.manifest.sheet_names.contains_key(&index) {
                        self.result.precedents.insert(index);
                    } else {
                        self.result
                            .blockers
                            .insert(DependencyBlocker::UnresolvedSheetReference);
                    }
                }
                self.walk(current_sheet, inner, locals);
            }
            ASTNode::UnresolvedSheetRef { inner, .. }
            | ASTNode::UnresolvedThreeDRef { inner, .. } => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::UnresolvedSheetReference);
                self.walk(current_sheet, inner, locals);
            }
            ASTNode::ExternalSheetRef { inner, .. } | ASTNode::ExternalThreeDRef { inner, .. } => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::ExternalReference);
                self.walk(current_sheet, inner, locals);
            }
            ASTNode::ExternalNameRef { .. } => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::ExternalReference);
            }
            ASTNode::StructuredRef(_) => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::UnsupportedStructuredReference);
            }
            ASTNode::Identifier(name) => {
                if !locals.iter().any(|local| local.eq_ignore_ascii_case(name)) {
                    self.resolve_name(current_sheet, name);
                }
            }
            ASTNode::Error(CellError::Ref) => {
                self.result
                    .blockers
                    .insert(DependencyBlocker::UnresolvedSheetReference);
            }
            ASTNode::Function { name, args } => {
                if name.eq_ignore_ascii_case("INDIRECT") {
                    self.result
                        .blockers
                        .insert(DependencyBlocker::DynamicIndirect);
                }
                if name.eq_ignore_ascii_case("LET") {
                    self.walk_let(current_sheet, args, locals);
                } else if name.eq_ignore_ascii_case("LAMBDA") {
                    self.walk_lambda(current_sheet, args, locals);
                } else {
                    for arg in args {
                        self.walk(current_sheet, arg, locals);
                    }
                }
            }
            ASTNode::BinaryOp { left, right, .. } => {
                self.walk(current_sheet, left, locals);
                self.walk(current_sheet, right, locals);
            }
            ASTNode::UnaryOp { operand, .. } | ASTNode::Paren(operand) => {
                self.walk(current_sheet, operand, locals);
            }
            ASTNode::Array { rows } => {
                for row in rows {
                    for value in row {
                        self.walk(current_sheet, value, locals);
                    }
                }
            }
            ASTNode::CallExpression { callee, args } => {
                self.walk(current_sheet, callee, locals);
                for arg in args {
                    self.walk(current_sheet, arg, locals);
                }
            }
            ASTNode::RangeOp { start, end } => {
                self.walk(current_sheet, start, locals);
                self.walk(current_sheet, end, locals);
            }
            ASTNode::Union { ranges } => {
                for range in ranges {
                    self.walk(current_sheet, range, locals);
                }
            }
            ASTNode::Number(_)
            | ASTNode::Text(_)
            | ASTNode::Boolean(_)
            | ASTNode::Error(_)
            | ASTNode::CellReference(_)
            | ASTNode::Range(_)
            | ASTNode::OptionalLambdaParam(_)
            | ASTNode::Omitted => {}
        }
    }

    fn walk_let(&mut self, current_sheet: u32, args: &[ASTNode], outer: &BTreeSet<String>) {
        let mut locals = outer.clone();
        let binding_pairs = args.len().saturating_sub(1) / 2;
        for pair in 0..binding_pairs {
            let name_index = pair * 2;
            let value_index = name_index + 1;
            self.walk(current_sheet, &args[value_index], &locals);
            if let ASTNode::Identifier(name) = &args[name_index] {
                locals.insert(name.to_ascii_lowercase());
            }
        }
        if let Some(result) = args.last() {
            self.walk(current_sheet, result, &locals);
        }
    }

    fn walk_lambda(&mut self, current_sheet: u32, args: &[ASTNode], outer: &BTreeSet<String>) {
        let mut locals = outer.clone();
        if let Some((body, parameters)) = args.split_last() {
            for parameter in parameters {
                match parameter {
                    ASTNode::Identifier(name) | ASTNode::OptionalLambdaParam(name) => {
                        locals.insert(name.to_ascii_lowercase());
                    }
                    other => self.walk(current_sheet, other, outer),
                }
            }
            self.walk(current_sheet, body, &locals);
        }
    }

    fn resolve_name(&mut self, current_sheet: u32, name: &str) {
        let found = self
            .manifest
            .defined_names
            .iter()
            .find(|defined| {
                defined.scope_workbook_index == Some(current_sheet)
                    && defined.name.eq_ignore_ascii_case(name)
            })
            .or_else(|| {
                self.manifest.defined_names.iter().find(|defined| {
                    defined.scope_workbook_index.is_none()
                        && defined.name.eq_ignore_ascii_case(name)
                })
            });
        let Some(defined) = found else {
            self.result
                .blockers
                .insert(DependencyBlocker::UnresolvedNamedRange);
            return;
        };
        let key = (
            defined.scope_workbook_index,
            defined.name.to_ascii_lowercase(),
        );
        if self.name_stack.contains(&key) {
            self.result
                .blockers
                .insert(DependencyBlocker::CyclicNamedRange);
            return;
        }
        let name_sheet = defined.scope_workbook_index.unwrap_or(current_sheet);
        let expression = defined.refers_to.clone();
        self.name_stack.push(key);
        self.analyze_expression(name_sheet, &expression, &BTreeSet::new());
        self.name_stack.pop();
    }
}

pub(super) fn parse_sheet_dependency_manifest_impl(
    xlsx_data: &[u8],
) -> Result<SheetDependencyManifest, String> {
    if xlsx_data.is_empty() {
        return Err("Empty XLSX data".to_string());
    }
    let archive =
        XlsxArchive::new(xlsx_data).map_err(|e| format!("Failed to open XLSX archive: {e}"))?;
    let workbook_xml = archive
        .get_workbook()
        .map_err(|e| format!("Failed to read xl/workbook.xml: {e}"))?;
    let workbook_relationships = match archive.read_file("xl/_rels/workbook.xml.rels") {
        Ok(xml) => workbook::parse_all_rels(&xml),
        Err(ZipError::FileNotFound(_)) => Vec::new(),
        Err(e) => return Err(format!("Failed to read xl/_rels/workbook.xml.rels: {e}")),
    };
    let content_types = archive
        .get_content_types()
        .ok()
        .and_then(|xml| ContentTypes::parse(&xml).ok());
    let inventory = workbook::build_workbook_sheet_inventory(
        &workbook::parse_workbook(&workbook_xml),
        &workbook_relationships,
        content_types.as_ref(),
        &archive,
    );

    let mut manifest = SheetDependencyManifest {
        sheet_names: inventory
            .iter()
            .map(|entry| (entry.workbook_order, entry.name.clone()))
            .collect(),
        defined_names: DefinedNames::parse(&workbook_xml)
            .iter()
            .filter(|defined| defined.built_in_type().is_none())
            .map(|defined| DependencyDefinedName {
                name: defined.name.clone(),
                refers_to: defined.refers_to.clone(),
                scope_workbook_index: defined.local_sheet_id,
            })
            .collect(),
        ..SheetDependencyManifest::default()
    };

    for entry in inventory
        .iter()
        .filter(|entry| entry.editable_sheet_index.is_some())
    {
        let owner = entry.workbook_order;
        let Some(path) = entry.normalized_part_path.as_deref() else {
            manifest
                .blockers_by_dependent
                .entry(owner)
                .or_default()
                .insert(DependencyBlocker::MalformedFormula);
            continue;
        };
        let xml = archive
            .read_file(path)
            .map_err(|e| format!("Failed to read worksheet {path}: {e}"))?;
        let formulas = match scan_formula_text(&xml) {
            Ok(formulas) => formulas,
            Err(()) => {
                manifest
                    .blockers_by_dependent
                    .entry(owner)
                    .or_default()
                    .insert(DependencyBlocker::MalformedFormula);
                continue;
            }
        };
        for formula in formulas {
            let analysis = manifest.analyze_formula(owner, &formula);
            manifest
                .precedents_by_dependent
                .entry(owner)
                .or_default()
                .extend(analysis.precedents);
            manifest
                .blockers_by_dependent
                .entry(owner)
                .or_default()
                .extend(analysis.blockers);
        }
    }
    manifest
        .precedents_by_dependent
        .retain(|_, precedents| !precedents.is_empty());
    manifest
        .blockers_by_dependent
        .retain(|_, blockers| !blockers.is_empty());
    Ok(manifest)
}

fn scan_formula_text(xml: &[u8]) -> Result<Vec<String>, ()> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut formulas = Vec::new();
    let mut formula = None::<String>;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) if event.local_name().as_ref() == b"f" => {
                formula = Some(String::new());
            }
            Ok(Event::End(event)) if event.local_name().as_ref() == b"f" => {
                if let Some(formula) = formula.take().filter(|formula| !formula.is_empty()) {
                    formulas.push(formula);
                }
            }
            Ok(Event::Text(text)) if formula.is_some() => {
                formula
                    .as_mut()
                    .expect("formula was checked above")
                    .push_str(&text.unescape().map_err(|_| ())?);
            }
            Ok(Event::CData(text)) if formula.is_some() => {
                formula
                    .as_mut()
                    .expect("formula was checked above")
                    .push_str(&String::from_utf8_lossy(text.as_ref()));
            }
            Ok(Event::Eof) if formula.is_some() => return Err(()),
            Ok(Event::Eof) => break,
            Err(_) => return Err(()),
            _ => {}
        }
    }
    Ok(formulas)
}
