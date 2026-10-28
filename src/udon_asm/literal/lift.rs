//! Strict literal decoding for UdonSharp. The dc/dasm renderer stays unchanged.
use super::enum_map::enum_repr;
use super::{HeapLiteralValue, literal_from_typed_odin_node, render_heap_literal, type_name_head};
use crate::odin::{NodeKind, PrimitiveValue, UdonProgramBinary};

pub(crate) fn render_lift_heap_entry(
    program: &UdonProgramBinary,
    index: usize,
) -> Result<String, String> {
    let type_name = program
        .heap_dump_type_name_string(index)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "unsupported constant <unknown>".to_string())?;
    let unsupported = || format!("unsupported constant {}", type_name_head(&type_name));
    let id = program
        .heap_dump_strongbox_value_node_id(index)
        .map_err(|e| e.to_string())?;
    let doc = program.document();
    let resolved = doc.resolve_node_payload(id).unwrap_or(id);
    let node = doc.node(resolved).ok_or_else(unsupported)?;
    let literal = literal_from_typed_odin_node(doc, id, &type_name);
    let head = type_name_head(&type_name);
    // Odin stores enum values as integral primitives, retaining the enum type in the heap.
    // Keep the typed value here instead of relying on dc's small table of member names.
    if enum_repr(head).is_some() || matches!(literal, HeapLiteralValue::Unserializable) {
        let value = match node.kind() {
            NodeKind::Primitive(PrimitiveValue::SByte(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::Byte(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::Short(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::UShort(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::Int(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::UInt(v)) => Some(v.to_string()),
            NodeKind::Primitive(PrimitiveValue::Long(v)) => Some(format!("{v}L")),
            NodeKind::Primitive(PrimitiveValue::ULong(v)) => Some(if *v <= i32::MAX as u64 {
                v.to_string()
            } else {
                format!("{v}UL")
            }),
            _ => None,
        };
        if let Some(value) = value {
            return Ok(format!("(global::{}){}", head.replace('+', "."), value));
        }
    }
    if head == "UnityEngine.Vector4"
        && let Some((x, y, z, w)) = super::parse::extract_quaternion(doc, resolved)
    {
        let components =
            [x, y, z, w].map(|v| render_heap_literal("System.Single", &HeapLiteralValue::F32(v)));
        return Ok(format!(
            "new global::UnityEngine.Vector4({})",
            components.join(", ")
        ));
    }
    strict_literal(&type_name, &literal)
}

fn strict_literal(type_name: &str, value: &HeapLiteralValue) -> Result<String, String> {
    match value {
        HeapLiteralValue::OpaqueArray { len: 0 } if type_name_head(type_name).ends_with("[]") => {
            Ok(format!(
                "new global::{}[0]",
                type_name_head(type_name)
                    .trim_end_matches("[]")
                    .replace('+', ".")
            ))
        }
        HeapLiteralValue::Unserializable | HeapLiteralValue::OpaqueArray { .. } => Err(format!(
            "unsupported constant {}",
            type_name_head(type_name)
        )),
        HeapLiteralValue::TypedArray {
            element_type,
            elements,
        } => {
            let values = elements
                .iter()
                .map(|v| strict_literal(element_type, v))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!(
                "new global::{}[] {{ {} }}",
                element_type.replace('+', "."),
                values.join(", ")
            ))
        }
        _ => {
            let text = render_heap_literal(type_name, value);
            if let Some(rest) = text.strip_prefix("new ") {
                Ok(format!("new global::{rest}"))
            } else if let Some(rest) = text.strip_prefix("typeof(") {
                Ok(format!("typeof(global::{rest}"))
            } else {
                Ok(text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn artsy_pickup_hand_values_survive_without_changing_pseudo_source() {
        let hex = include_str!("../../../tests/fixtures/ArtsyToggleMultiple.hex");
        let bytes = crate::util::decode_compressed_hex_text(hex).unwrap();
        let program = UdonProgramBinary::parse_bytes(&bytes).unwrap();
        let mut values = Vec::new();
        for i in 0..program.heap_dump_len().unwrap() {
            let ty = program.heap_dump_type_name_string(i).unwrap().unwrap();
            if ty.starts_with("VRC.SDKBase.VRC_Pickup+PickupHand,") {
                values.push(render_lift_heap_entry(&program, i).unwrap());
                let id = program.heap_dump_strongbox_value_node_id(i).unwrap();
                let old = literal_from_typed_odin_node(program.document(), id, &ty);
                assert_eq!(
                    render_heap_literal(&ty, &old),
                    crate::str_constants::UNSERIALIZABLE_LITERAL
                );
            }
        }
        assert_eq!(
            values,
            [
                "(global::VRC.SDKBase.VRC_Pickup.PickupHand)1",
                "(global::VRC.SDKBase.VRC_Pickup.PickupHand)2"
            ]
        );
    }
    #[test]
    fn opaque_values_fail_explicitly_including_inside_arrays() {
        assert_eq!(
            strict_literal("Example.Unknown", &HeapLiteralValue::Unserializable).unwrap_err(),
            "unsupported constant Example.Unknown"
        );
        let array = HeapLiteralValue::TypedArray {
            element_type: "Example.Unknown".into(),
            elements: vec![HeapLiteralValue::Unserializable],
        };
        assert!(strict_literal("Example.Unknown[]", &array).is_err());
        assert_eq!(
            strict_literal(
                "UnityEngine.GameObject[]",
                &HeapLiteralValue::OpaqueArray { len: 0 }
            )
            .unwrap(),
            "new global::UnityEngine.GameObject[0]"
        );
    }
    #[test]
    fn supported_structs_keep_every_component() {
        for (ty, value, expected) in [
            (
                "UnityEngine.Vector2",
                HeapLiteralValue::Vector2(1.0, 2.0),
                "new global::UnityEngine.Vector2(1.0f, 2.0f)",
            ),
            (
                "UnityEngine.Vector3",
                HeapLiteralValue::Vector3(1.0, 2.0, 3.0),
                "new global::UnityEngine.Vector3(1.0f, 2.0f, 3.0f)",
            ),
            (
                "UnityEngine.Quaternion",
                HeapLiteralValue::Quaternion(1.0, 2.0, 3.0, 4.0),
                "new global::UnityEngine.Quaternion(1.0f, 2.0f, 3.0f, 4.0f)",
            ),
            (
                "UnityEngine.Color",
                HeapLiteralValue::Color(1.0, 2.0, 3.0, 4.0),
                "new global::UnityEngine.Color(1.0f, 2.0f, 3.0f, 4.0f)",
            ),
        ] {
            assert_eq!(strict_literal(ty, &value).unwrap(), expected);
        }
    }
}
