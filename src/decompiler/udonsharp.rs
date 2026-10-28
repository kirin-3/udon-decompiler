use super::{DecompileContext, DecompileError, Result};
use crate::odin::{NodeId, NodeKind, OdinDocument, OdinTypeRef, PrimitiveValue, UdonProgramBinary};
use crate::udon_asm::render_lift_heap_entry;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct UdonSharpArgument {
    pub symbol: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct UdonSharpEvent {
    pub method_name: String,
    pub udon_name: String,
    pub arguments: Vec<UdonSharpArgument>,
    pub is_override: bool,
}
#[derive(Debug, Clone)]
pub struct UdonSharpMetadata {
    pub events: BTreeMap<String, UdonSharpEvent>,
    pub synced_fields: BTreeMap<String, String>,
    pub behaviour_sync_mode: Option<String>,
    pub parameter_symbols: HashSet<String>,
    pub enum_constants: HashSet<u32>,
}

impl DecompileContext {
    pub fn enable_udonsharp(
        &mut self,
        program: &UdonProgramBinary,
        events_path: &Path,
        sync_mode: Option<String>,
    ) -> Result<()> {
        if self.inferred_program_class_name().is_none() {
            return Err(DecompileError::new("no UdonSharp class identity"));
        }
        if sync_mode.as_deref().is_some_and(|mode| {
            !["None", "NoVariableSync", "Continuous", "Manual", "Any"].contains(&mode)
        }) {
            return Err(DecompileError::new("unsupported behaviour sync mode"));
        }
        let text = std::fs::read_to_string(events_path).map_err(|e| {
            DecompileError::new(format!("event table {}: {e}", events_path.display()))
        })?;
        let events: BTreeMap<String, UdonSharpEvent> =
            serde_json::from_str(&text).map_err(|e| DecompileError::new(e.to_string()))?;
        let parameter_symbols: HashSet<String> = self
            .entry_points
            .iter()
            .filter_map(|entry| events.get(&entry.name))
            .flat_map(|event| event.arguments.iter().map(|arg| arg.symbol.clone()))
            .collect();
        // Only this context gets strict literals. dc and dasm retain their original decoder.
        let mut enum_constants = HashSet::new();
        for i in 0..program.heap_dump_len()? {
            let address = program.heap_dump_item(i)?.address;
            // Self references are runtime bindings, not heap constants. VariableTable renders them as this/gameObject/transform.
            if self.symbols.iter().any(|symbol| {
                symbol.address == address
                    && (symbol.name.starts_with("__this_")
                        || parameter_symbols.contains(&symbol.name))
            }) {
                continue;
            }
            let value = render_lift_heap_entry(program, i).map_err(DecompileError::new)?;
            if value.starts_with("(global::")
                && self
                    .symbols
                    .iter()
                    .any(|symbol| symbol.address == address && symbol.name.starts_with("__const_"))
            {
                enum_constants.insert(address);
            }
            if let Some(entry) = self
                .heap_entries
                .iter_mut()
                .find(|entry| entry.address == address)
            {
                entry.init_value = value;
            }
        }
        self.udonsharp = Some(UdonSharpMetadata {
            events,
            synced_fields: sync_fields(program.document())?,
            behaviour_sync_mode: sync_mode,
            parameter_symbols,
            enum_constants,
        });
        Ok(())
    }
}

fn descendants(doc: &OdinDocument, root: NodeId) -> Vec<NodeId> {
    let mut pending = vec![root];
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    while let Some(id) = pending.pop() {
        let id = doc.resolve_node_payload(id).unwrap_or(id);
        if !seen.insert(id) {
            continue;
        }
        if let Some(node) = doc.node(id) {
            result.push(id);
            pending.extend(node.children().iter().rev().copied());
        }
    }
    result
}
fn sync_fields(doc: &OdinDocument) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for node in doc.nodes() {
        let name = match node.kind() {
            NodeKind::ReferenceNode {
                type_ref: OdinTypeRef::TypeName { name, .. },
                ..
            } => Some(name.value.as_str()),
            NodeKind::ReferenceNode {
                type_ref: OdinTypeRef::TypeId { resolved_name, .. },
                ..
            } => resolved_name.as_deref(),
            _ => None,
        };
        if !name.is_some_and(|name| name.starts_with("VRC.Udon.Common.UdonSyncMetadata,")) {
            continue;
        }
        let nodes = descendants(doc, node.id());
        let field = nodes
            .iter()
            .filter_map(|id| doc.node(*id))
            .find_map(|n| match n.kind() {
                NodeKind::Primitive(PrimitiveValue::String(value))
                    if n.name().is_some_and(|s| s.value == "Name") =>
                {
                    Some(value.value.clone())
                }
                _ => None,
            })
            .ok_or_else(|| DecompileError::new("sync metadata has no variable name"))?;
        let mut modes = HashSet::new();
        for n in nodes.iter().filter_map(|id| doc.node(*id)).filter(|n| {
            n.name()
                .is_some_and(|s| s.value == "InterpolationAlgorithm")
        }) {
            let mode = match n.kind() {
                NodeKind::Primitive(PrimitiveValue::ULong(0)) => "None",
                NodeKind::Primitive(PrimitiveValue::ULong(1)) => "Linear",
                NodeKind::Primitive(PrimitiveValue::ULong(2)) => "Smooth",
                _ => {
                    return Err(DecompileError::new(format!(
                        "unsupported sync interpolation for {field}"
                    )));
                }
            };
            modes.insert(mode);
        }
        if modes.len() != 1 {
            return Err(DecompileError::new(format!(
                "unsupported sync properties for {field}"
            )));
        }
        result.insert(field, modes.into_iter().next().unwrap().to_string());
    }
    Ok(result)
}

pub(super) fn identifier(name: &str) -> String {
    if [
        "abstract",
        "as",
        "base",
        "bool",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "checked",
        "class",
        "const",
        "continue",
        "decimal",
        "default",
        "delegate",
        "do",
        "double",
        "else",
        "enum",
        "event",
        "explicit",
        "extern",
        "false",
        "finally",
        "fixed",
        "float",
        "for",
        "foreach",
        "goto",
        "if",
        "implicit",
        "in",
        "int",
        "interface",
        "internal",
        "is",
        "lock",
        "long",
        "namespace",
        "new",
        "null",
        "object",
        "operator",
        "out",
        "override",
        "params",
        "private",
        "protected",
        "public",
        "readonly",
        "ref",
        "return",
        "sbyte",
        "sealed",
        "short",
        "sizeof",
        "stackalloc",
        "static",
        "string",
        "struct",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "uint",
        "ulong",
        "unchecked",
        "unsafe",
        "ushort",
        "using",
        "virtual",
        "void",
        "volatile",
        "while",
    ]
    .contains(&name)
    {
        format!("@{name}")
    } else {
        name.to_string()
    }
}
pub(super) fn local_name(name: &str, type_name: &str) -> String {
    let Some(body) = name.strip_prefix("__lcl_") else {
        return identifier(name);
    };
    let body = body.rsplit_once('_').map_or(body, |(body, _)| body);
    let ty = type_name
        .split(',')
        .next()
        .unwrap_or(type_name)
        .replace("[]", "Array")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>();
    let source = body
        .strip_suffix(&format!("_{ty}"))
        .unwrap_or_else(|| body.rsplit_once('_').map_or(body, |(name, _)| name));
    identifier(source)
}
